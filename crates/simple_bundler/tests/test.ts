import { a, add, addAsync } from "./utils";

console.log(add(1, 2));

addAsync(1, 2).then((res) => console.log(res));

console.log(a);
