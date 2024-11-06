export function add(a: number, b: number) {
  return a + b;
}

export function addAsync(a: number, b: number) {
  return new Promise((resolve) => {
    setTimeout(() => resolve(a + b), 1000);
  });
}

export const a = 1;