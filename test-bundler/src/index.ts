import { sum } from 'lodash-es'

export function getSum() {
  return sum([1, 2, 3])
}

export function asyncGetSum() {
  return new Promise(resolve => {
    setTimeout(() => {
      resolve(getSum())
    }, 1000)
  })
}
