export function sleep(m: number = 0) {
  return new Promise((resolve) => {
    setTimeout(resolve, m);
  });
}

export async function asyncHello(): Promise<string> {
    return Promise.resolve("hello async");
}

export function intervalCount(ms: number, count: number): Promise<number> {
    return new Promise((resolve) => {
        let c = 0;
        const id = setInterval(() => {
            c++;
            if (c >= count) {
                clearInterval(id);
                resolve(c);
            }
        }, ms);
    });
}
