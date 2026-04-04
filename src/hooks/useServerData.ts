import { useCallback, useEffect, useRef, useState } from "react";
import type { OnChangeFn, SortingState } from "@tanstack/react-table";
import type { PaginatedResponse, PaginationParams } from "@/types";

const DEFAULT_PAGE_SIZE = 100;

interface UseServerDataOptions<F> {
  filters: F;
  initialSortBy?: string;
  initialSortDesc?: boolean;
  pageSize?: number;
}

interface UseServerDataResult<T> {
  items: T[];
  total: number;
  isLoading: boolean;
  isLoadingMore: boolean;
  hasMore: boolean;
  error: string | null;
  loadMore: () => void;
  sorting: SortingState;
  setSorting: OnChangeFn<SortingState>;
}

export function useServerData<T, F>(
  fetcher: (pagination: PaginationParams, filters: F) => Promise<PaginatedResponse<T>>,
  options: UseServerDataOptions<F>,
): UseServerDataResult<T> {
  const {
    filters,
    initialSortBy,
    initialSortDesc = false,
    pageSize = DEFAULT_PAGE_SIZE,
  } = options;

  const [items, setItems] = useState<T[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sorting, setSortingState] = useState<SortingState>(
    initialSortBy ? [{ id: initialSortBy, desc: initialSortDesc }] : [],
  );

  const inFlightRef = useRef(0);
  const filtersRef = useRef(filters);
  const sortingRef = useRef(sorting);

  filtersRef.current = filters;
  sortingRef.current = sorting;

  const buildPagination = useCallback(
    (offset: number): PaginationParams => {
      const sort = sortingRef.current[0];
      return {
        offset,
        limit: pageSize,
        sortBy: sort?.id ?? null,
        sortDir: sort ? (sort.desc ? "desc" : "asc") : null,
      };
    },
    [pageSize],
  );

  // Initial fetch and refetch on filter/sort change
  useEffect(() => {
    const requestId = ++inFlightRef.current;
    setIsLoading(true);
    setError(null);

    fetcher(buildPagination(0), filtersRef.current)
      .then((res) => {
        if (inFlightRef.current !== requestId) return;
        setItems(res.items);
        setTotal(res.total);
      })
      .catch((err) => {
        if (inFlightRef.current !== requestId) return;
        setError(String(err));
        setItems([]);
        setTotal(0);
      })
      .finally(() => {
        if (inFlightRef.current !== requestId) return;
        setIsLoading(false);
      });
  }, [fetcher, filters, sorting, buildPagination]);

  const hasMore = items.length < total;

  const loadMore = useCallback(() => {
    if (isLoadingMore || isLoading || !hasMore) return;
    const requestId = ++inFlightRef.current;
    setIsLoadingMore(true);

    fetcher(buildPagination(items.length), filtersRef.current)
      .then((res) => {
        if (inFlightRef.current !== requestId) return;
        setItems((prev) => [...prev, ...res.items]);
        setTotal(res.total);
      })
      .catch((err) => {
        if (inFlightRef.current !== requestId) return;
        setError(String(err));
      })
      .finally(() => {
        if (inFlightRef.current !== requestId) return;
        setIsLoadingMore(false);
      });
  }, [fetcher, buildPagination, items.length, isLoadingMore, isLoading, hasMore]);

  const setSorting: OnChangeFn<SortingState> = useCallback((updaterOrValue) => {
    setSortingState((prev) =>
      typeof updaterOrValue === "function" ? updaterOrValue(prev) : updaterOrValue,
    );
  }, []);

  return {
    items,
    total,
    isLoading,
    isLoadingMore,
    hasMore,
    error,
    loadMore,
    sorting,
    setSorting,
  };
}
