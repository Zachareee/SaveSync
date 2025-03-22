export type OsString = Record<"Windows" | "Unix", number[]>
export type SystemTime = Record<`${"nanos" | "secs"}_since_epoch`, number>
