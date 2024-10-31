export type Result<T> = [NonNullable<T>, null | Error];


export type PR<T> = Promise<Result<T>>;