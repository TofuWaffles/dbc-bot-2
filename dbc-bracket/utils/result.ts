export type Result<T> = [NonNullable<T>, null | Error];
export const Ok = <T>(value: NonNullable<T>): Result<T> => [value, null];
export const Err = <T>(error: Error): Result<T> => [undefined, error];

 /**
     * Retrieves the value from a Result, throwing an error if the result indicates failure.
     *
     * @param result - A Result<T> tuple where the first element is the successful value
     *                 of type NonNullable<T>, and the second element is either null or an Error.
     * 
     * @param message - An optional message to include in the error if the result indicates failure.
     * 
     * @returns The successful value of type T if the result indicates success.
     * 
     * @throws Error - Throws the error contained in the result if the result indicates failure.
     * 
     **/
export function expect<T>(result: Result<T>, message?: string): NonNullable<T> {
    if (result[1]) {
        throw new Error(message ?? result[1].message);
    }
    return result[0];
}

export type PR<T> = Promise<Result<T>>;