export type Recipe = {
  items: Item[];
  total: Mix;
  mixes: Mix[];
  final: number[];
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

export const TOTAL_FLOUR: Item = { name: "total flour", is_flour: false };

export const TOTAL: Item = { name: "total", is_flour: false };

export const Mix = (): Mix => ({
  name: "",
  amounts: [],
  flour: Amounts(),
  total: Amounts(),
});

export const Amounts = (): Amounts => ({ weight: 0.0, bakers: 0.0, in_other: 0.0 });

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
    name: "pizza 🍕",
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
  final: Array(6).fill(0),
});
