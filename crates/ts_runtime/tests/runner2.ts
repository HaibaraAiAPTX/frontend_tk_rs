import { sleep } from "./utils";

sleep().then(() => {
  globalThis.result = "hello world async result";
});
