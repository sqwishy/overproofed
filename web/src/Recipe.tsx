export type V0 = V0.Recipe;
export type V1 = { v: 1 } & Recipe;
export type HashDeserializable = V0 | V1;
export type HashSerialized = V1;

export function serializeToString(recipe: Recipe) {
  const toSave: HashSerialized = { v: 1, ...recipe};
  return JSON.stringify(toSave);
}

export function deserializeFromString(s: string) {
  const value = JSON.parse(s);

  /* todo error checking and handling */
  if (value.v === undefined) {
    const zeroToNull = (v: number) => v === 0 ? null : v;
    const v0ZMapAmounts =
      ({ weight, bakers, in_other }: V0.Amounts) =>
      ({ weight: zeroToNull(weight), bakers: zeroToNull(bakers), in_other: zeroToNull(in_other) });
    const v0MapMix =
      ({ name, amounts, total, flour }: V0.Mix) =>
      ({ name, amounts: amounts.map((a) => a === null ? null : v0ZMapAmounts(a)), total: v0ZMapAmounts(total), flour: v0ZMapAmounts(flour) })

    const { items, total, mixes, final }: V0.Recipe = value;
    return {
      items,
      total: v0MapMix(total),
      mixes: mixes.map(v0MapMix),
      final: final.map(zeroToNull),
    }

  } else {
    return value

  }
}

export type Recipe = {
  items: Item[];
  total: Mix;
  mixes: Mix[];
  final: (number | null)[];
};

export type Item = {
  name: string;
  is_flour: boolean;
};

export type Mix = {
  name: string;
  amounts: (Amounts | null)[];
  flour: Amounts;
  total: Amounts;
};

export type Amounts = {
  weight: number | null;
  bakers: number | null;
  in_other: number | null;
};

export const TOTAL_FLOUR: Item = { name: "total flour", is_flour: false };

export const TOTAL: Item = { name: "total", is_flour: false };

export const Mix = (): Mix => ({
  name: "",
  amounts: [],
  flour: Amounts(),
  total: Amounts(),
});

export const Amounts = (): Amounts => ({ weight: null, bakers: null, in_other: null });

export const defaultRecipe = () => ({
  items: [
    { name: "rye", is_flour: true },
    { name: "wheat", is_flour: true },
    { name: "00", is_flour: true },
    { name: "vital wheat gluten", is_flour: true },
    { name: "water 🚿", is_flour: false },
    { name: "salt 🧂", is_flour: false },
  ],
  total: {
    ...Mix(),
    name: "pizza 🍕 one ball = 240g",
    amounts: [
      { ...Amounts(), in_other: 1.0 },
      { ...Amounts() },
      { ...Amounts(), bakers: 0.20 },
      { ...Amounts(), bakers: 0.06 },
      { ...Amounts(), bakers: 0.72 },
      { ...Amounts(), bakers: 0.027 },
    ],
    total: { ...Amounts(), weight: 960 },
    flour: { ...Amounts(), in_other: 0.1 },
  },
  mixes: [
    {
      ...Mix(),
      name: "soggy rye starter 💦",
      amounts: [Amounts(), null, null, null, { ...Amounts(), bakers: 2.0 }, null],
    },
  ],
  final: Array(6).fill(null),
});

namespace V0 {
  export type Recipe = {
    items: Item[];
    total: Mix;
    mixes: Mix[];
    final: (number)[];
  };

  export type Item = {
    name: string;
    is_flour: boolean;
  };

  export type Mix = {
    name: string;
    amounts: (Amounts | null)[];
    flour: Amounts;
    total: Amounts;
  };

  export type Amounts = {
    weight: number;
    bakers: number;
    in_other: number;
  };
}
