import { Result } from "./result";

Array.prototype.get = function<T>(this: T[], index: number): Result<T> {
  const len = this.length;
  if (index < 0) {
    index += len; 
  }
  if (index < 0 || index >= len) {
    return [null, new Error(`Index ${index} is out of bounds for length ${len}`)];
  }
  return [this[index], null]; 
};

export{};