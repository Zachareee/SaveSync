export type Info = Record<"name" | "description" | "author" | "file_name", string> & Partial<
  Record<"icon_url", string>
>;
