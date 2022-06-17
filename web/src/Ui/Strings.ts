export function isZero(v: any) {
   return v === undefined || v === 0 || Number.isNaN(v);
}

export function isNonZero(v: any) {
  return !isZero(v);
}

export function percent(f: number) {
  return (100 * f).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';
}

// export function kg(grams: number) {
//   return (grams / 1000).toLocaleString(undefined, {minimumFractionDigits: 3, maximumFractionDigits: 3});
// }

// export function number(n: number) {
//   return n.toLocaleString(undefined, {maximumFractionDigits: 3});
// }

export function grams(n: number) {
  // FIXME use user agent locale when parsing supports it
  return n.toLocaleString("en", { maximumFractionDigits: 0 }) + 'g';
}

export function parseNum(s: string) {
  // FIXME not locale aware?
  const n = Number(s.replaceAll(',', ''));
  return isFinite(n) ? n : 0;
}

export function parsePercent(s: string) {
  let match;
  if (match = s.trimEnd().match(/([\d,.]*) *%$/)) {
    return parseNum(match[1]) / 100;
  } else {
    return parseNum(s) / 100;
  }
}

export function parseWeightToGrams(s: string) {
  let match;
  if (match = s.trimEnd().match(/([\d,.]*) *(k?g?)?/)) {
    const [, num, units] = match;
    return parseNum(num) * ((units === 'kg' || units === 'k') ? 1000 : 1);
  } else {
    return parseNum(s);
  }
}
