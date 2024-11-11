export function sleep(m: number = 0) {
  return new Promise((resolve) => {
    setTimeout(resolve, m);
  });
}