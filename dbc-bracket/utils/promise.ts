import { Result } from "./result";

// Promise.prototype.wrapper = function <T>(): Result<T> {
//   let result: Result<T> = [null, null];
//   this.then((data: T) => {
//     result[0] = data;
//   }).catch((error: Error) => {
//     result[1] = error;
//   });
//   console.debug("After resolving promise: ",result);
//   return result;
// };
Promise.prototype.wrapper = async function <T>(): Promise<Result<T>> {
  try {
    const value: T = await this;
    return [value, null];
  } catch (error) {
    return [null, error];
  }
};

Promise.prototype.unwrap = function <T>() {
  return this.then((value: T) => value).catch((error) => {
    throw error;
  });
};

Promise.prototype.unwrapOr = async function <T>(defaultValue: T) {
  try {
    const value = await this;
    return value;
  } catch (error) {
    return defaultValue;
  }
};

export {};
