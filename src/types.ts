export type Info = Record<"name" | "description" | "author", string> & Partial<
  Record<"icon_url", string>
>;
