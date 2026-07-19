import { DataTable } from "@/components/data-table";
import { api, RoleResponse } from "@/lib/api";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import * as z from "zod";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Skeleton } from "@/components/ui/skeleton";
import { ColumnDef } from "@tanstack/react-table";
import { Button } from "@/components/ui/button";
import { MoreHorizontal } from "lucide-react";

const roleSearchSchema = z.object({
  page: z.number().default(1),
  page_size: z.number().default(10),
});

export const Route = createFileRoute("/_admin/roles")({
  component: RouteComponent,
  validateSearch: roleSearchSchema,
});

export const columns: ColumnDef<RoleResponse>[] = [
  {
    accessorKey: "name",
    header: () => <TableHead className="w-1/2">Name</TableHead>,
  },
  {
    accessorKey: "description",
    header: () => <TableHead className="w-1/4">Description</TableHead>,
  },

  {
    accessorKey: "status",
    header: () => <TableHead className="w-1/4">Status</TableHead>,
  },
  {
    id: "actions",
    header: () => <TableHead className="w-1/4"></TableHead>,
    cell: ({ row }) => {
      const payment = row.original;

      return (
        <DropdownMenu>
          <DropdownMenuTrigger
            render={<Button variant="secondary" className="h-8 w-8 p-0" />}
          >
            <span className="sr-only">Open menu</span>
            <MoreHorizontal className="h-4 w-4" />
          </DropdownMenuTrigger>
          <DropdownMenuContent className="w-44" align="end">
            <DropdownMenuGroup>
              <DropdownMenuLabel>Actions</DropdownMenuLabel>
              <DropdownMenuItem
                onClick={() => navigator.clipboard.writeText(payment.id)}
              >
                Copy payment ID
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem>View customer</DropdownMenuItem>
              <DropdownMenuItem variant="destructive">
                Vô hiệu hoá
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];

function RouteComponent() {
  const { page, page_size } = Route.useSearch();

  const { isPending, data } = useQuery({
    queryKey: ["roles", { page, page_size }],
    queryFn: async () => await api.listRoles({ page, pageSize: page_size }),
    placeholderData: keepPreviousData,
  });

  return (
    <div className="container mx-auto p-4">
      <div className="card"></div>
      {isPending ? (
        <div className="overflow-hidden rounded-md border bg-card">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-1/2">
                  <Skeleton className="h-4 w-20" />
                </TableHead>
                <TableHead className="w-1/4">
                  <Skeleton className="h-4 w-20" />
                </TableHead>
                <TableHead className="w-1/4">
                  <Skeleton className="h-4 w-20" />
                </TableHead>
                <TableHead className="w-1/4">
                  <Skeleton className="h-4 w-20" />
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {Array.from({ length: 5 }).map((_, index) => (
                <TableRow key={index}>
                  <TableCell className="font-medium">
                    <Skeleton className="h-4 w-48" />
                  </TableCell>
                  <TableCell>
                    <Skeleton className="h-4 w-24" />
                  </TableCell>
                  <TableCell>
                    <Skeleton className="h-4 w-24" />
                  </TableCell>
                  <TableCell className="text-left">
                    <Skeleton className="h-4 w-24" />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      ) : (
        <DataTable
          columns={columns}
          data={data?.items ?? []}
          className="bg-card"
          page={page}
          total={data?.total ?? 0}
          pageSize={page_size}
        />
      )}
    </div>
  );
}
