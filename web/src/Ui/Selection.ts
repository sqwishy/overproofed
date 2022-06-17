import { createSignal } from "solid-js";
import { unwrap } from "solid-js/store";
import type { Signal, Accessor, Setter } from "solid-js";

export type Selection<T extends object> = {
  _map: WeakMap<T, Signal<boolean>>,
  _getOrInsert: (_: T) => Signal<boolean>,
  includesAny: (_: T[]) => boolean,
  count: (_: T[]) => number,
  clear: (_: T[]) => void,
  selected: (_: T[]) => T[],
  // take: (_: T[]) => T[],
  take: (_: T) => boolean,
  swaps: (_: boolean) => (_: T) => boolean,
  includes: (_: T) => boolean,
  toggle: (_: T) => void,
  gets: (_: T) => Accessor<boolean>,
  sets: (_: T) => Setter<boolean>,
}

export const createSelection = <T extends object,>() => {
    let self: Selection<T>;
    return self = {
        _map: new WeakMap(),
        _getOrInsert: (item) => {
            item = unwrap(item);
            let signal;
            if ((signal = self._map.get(item)) === undefined) {
                signal = createSignal(false);
                self._map.set(item, signal);
            }
            return signal;
        },
        /* returns whether any items are selected */
        includesAny: (items) => items.some(i => self.includes(i)),
        /* count selected items */
        count: (items) => items.filter(i => self.includes(i)).length,
        /*  unselect all given items */
        clear: (items) => items.map(i => self.sets(i)(false)),
        /* filter `items` by selected */
        selected: (items) => items.filter(i => self.includes(i)),
        // take: (items) => items.filter(item => self.swaps(false)(item)),
        take: (item) => self.swaps(false)(item),
        /* sets items to the given value, returning their previous value */
        swaps: value => item => {
            const [gets, sets] = self._getOrInsert(item);
            const last = gets();
            sets(value);
            return last;
        },
        includes: (item) => self.gets(item)(),
        toggle: (item) => {
            const [gets, sets] = self._getOrInsert(item);
            sets(!gets());
        },
        /* returns the signal setter */
        sets: (item) => {
            const [_, sets] = self._getOrInsert(item);
            return sets;
        },
        /* returns the signal getter */
        gets: (item) => {
            const [gets, _] = self._getOrInsert(item);
            return gets;
        },
    }
};
