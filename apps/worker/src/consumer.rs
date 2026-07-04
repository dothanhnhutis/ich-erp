use futures_lite::stream::StreamExt;
use lapin::{
    Connection, ConnectionProperties, ExchangeKind,
    message::Delivery,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions,
        BasicRejectOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::{FieldTable, ShortString},
};
use shared::messaging::{EMAIL_EXCHANGE, EMAIL_ROUTING_KEY, EmailJob};

use crate::{
    config::WorkerConfig,
    email::{EmailError, EmailSender},
};

/// Kết nối, declare topology, rồi consume tới khi connection/channel hỏng.
/// Trả về = stream đóng bình thường; Err = lỗi kết nối → caller sẽ reconnect.
pub async fn run(cfg: &WorkerConfig, sender: &EmailSender) -> Result<(), lapin::Error> {
    let conn = Connection::connect(&cfg.rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // lapin 4.x nhận `ShortString` cho tên — convert 1 lần, tái dùng (ShortString là Clone).
    let exchange: ShortString = EMAIL_EXCHANGE.into();
    let routing_key: ShortString = EMAIL_ROUTING_KEY.into();
    let queue: ShortString = cfg.email_queue.as_str().into();

    // Topology idempotent — chạy nhiều lần vô hại. durable để sống sót broker restart.
    channel
        .exchange_declare(
            exchange.clone(),
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel
        .queue_declare(
            queue.clone(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel
        .queue_bind(
            queue.clone(),
            exchange,
            routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // Chỉ nhận tối đa `prefetch` message chưa-ack cùng lúc → chia tải công bằng, tránh ôm hết.
    channel
        .basic_qos(cfg.prefetch, BasicQosOptions::default())
        .await?;

    let mut consumer = channel
        .basic_consume(
            queue,
            "email-worker".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tracing::info!(queue = %cfg.email_queue, "worker bắt đầu consume email job");

    while let Some(delivery) = consumer.next().await {
        // Err ở đây = connection/channel rớt → thoát vòng để caller reconnect.
        let delivery = delivery?;
        handle(delivery, sender).await;
    }

    Ok(())
}

/// Xử lý 1 message: parse → gửi → ack/nack/reject theo bản chất lỗi.
async fn handle(delivery: Delivery, sender: &EmailSender) {
    // 1. Parse payload. Hỏng = poison message, retry vô ích → reject không requeue.
    let job: EmailJob = match serde_json::from_slice(&delivery.data) {
        Ok(job) => job,
        Err(e) => {
            tracing::error!(error = %e, "payload không parse được, reject (drop)");
            let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
            return;
        }
    };

    // 2. Gửi mail, phân loại lỗi.
    match sender.send(job).await {
        Ok(()) => {
            if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                tracing::error!(error = %e, "ack thất bại");
            }
        }
        // Lỗi vĩnh viễn (địa chỉ sai...) → reject, đừng requeue kẻo lặp vô hạn.
        Err(EmailError::Invalid(msg)) => {
            tracing::error!(reason = %msg, "email hỏng vĩnh viễn, reject (drop)");
            let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
        }
        // Lỗi tạm thời (SMTP down) → nack + requeue để thử lại sau.
        Err(EmailError::Transport(msg)) => {
            tracing::warn!(reason = %msg, "gửi thất bại tạm thời, requeue");
            let _ = delivery
                .nack(BasicNackOptions {
                    requeue: true,
                    multiple: false,
                })
                .await;
        }
    }
}
