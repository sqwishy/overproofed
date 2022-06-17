// const shiftModifier = event => !event.altKey && event.shiftKey && !event.ctrlKey && !event.metaKey;
// const ctrlModifier = event => !event.altKey && !event.shiftKey && event.ctrlKey && !event.metaKey;
const noModifiers = event => !event.altKey && !event.shiftKey && !event.ctrlKey && !event.metaKey;

export const rowKbNav = (e: KeyboardEvent) => {
  if (noModifiers(e)) {
    if    (e.key == "ArrowUp")   rowFocusNext(e, -1)
    else if (e.key == "ArrowDown") rowFocusNext(e,  1)
  }
};

const rowFocusNext = (e: KeyboardEvent, dir: number) => {
  if (!e.target) return;

  let tries = 2, /* can skip a tr sibling if finding an input to focus on doesn't work  */
      td = e.target as HTMLElement,
      tr = td.closest('.mix-item'),
      next,
      mix;

  while (tries--) {
    /* if we have a row, check for a sibling row */
    if (   (tr)
        && (tr = dir < 0 ? tr.previousElementSibling : tr.nextElementSibling));
    /* otherwise, look for a row in a sibling .mix */
    else if (   (mix = td.closest('.mix'))
             && (mix = dir < 0 ? mix.previousElementSibling : mix.nextElementSibling)
             && (tr = dir < 0 ? mix.querySelector('.mix-item:last-child') : mix.querySelector('.mix-item')));
    /* couldn't find a row, early exit */
    else break;

    /* doesn't work well when the flour toggle is focused
     * since the total rows have them disabled (and hidden) */
    if (   (next = childAtSameIndex(tr, td))
        && (next.matches(':enabled'))) {
      next.focus();
      next.select?.(); /* can't select buttons */
      e.preventDefault();
      return;
    }
  }
};

/// parent.children[ otherChild.parent.children.indexOf(otherChild) ]
const childAtSameIndex = (parent: Element, otherChild_: Element) => {
  let otherChild: Element | null = otherChild_;
  let child = parent.firstElementChild;

  while (child && (otherChild = otherChild.previousElementSibling))
    child = child.nextElementSibling;

  return child;
}
