import {
  batch,
  splitProps,
  createEffect,
  createMemo,
  createSignal,
  JSX,
  Show,
  For,
  Accessor,
  Setter,
} from "solid-js";
import { createStore, unwrap, reconcile } from "solid-js/store";

import {
  isZero,
  isNonZero,
  isNonNull,
  percent,
  parsePercent,
  grams,
  parseWeightToGrams,
} from "./Ui/Strings";
import { Selection, createSelection } from "./Ui/Selection";
import { rowKbNav } from "./Ui/Focus";
import * as Icon from "./icon";
import * as wasm from "../public/p/overproofed_wasm.js";
import { Amounts, Mix, Recipe, Item, defaultRecipe, TOTAL, TOTAL_FLOUR, serializeToString, deserializeFromString } from "./Recipe";
import { deepcopy, last, dropLast, appends, setrattr, donotwant, removesAt, getsAt, insertsAt, nullIfEmpty, coalesce } from "./Halp";

const UNDO_LENGTH = 64;

/* several of these messages operate on the selection state, so not really totally self-describing :sadface: */
type Update =
  | "undo"
  | "redo"
  | "new-item"
  | "new-mix"
  | "remove-recipe-items"
  | { "remove-mix": number }
  | { "update-mix": MixTableUpdate; mix: "total" | number }
  | { "update-final": number, weight: number | null }
  | { "update-mix-name": string; mix: "total" | number }
  | { "add-mix-items": number }
  | { "remove-mix-items": number };

export type BreadProps = {
  history: Accessor<{ state: any; url: URL }>;
  title?: Setter<string>;
};

type Pair<A, B = A> = [A, B];

const Pair = {
  of: <T,>(a: T, b: T): Pair<T> => [a, b],
  map: <T, S>([a, b]: Pair<T>, fn: (_: T) => S): Pair<S> => [fn(a), fn(b)],
  firstNonZero: <T,>([a, b]: Pair<T>): T => (isNonZero(a) ? a : b),
  // filter: <A,B>([a, b]: Pair<A,B>, fn: (_: A | B) => Boolean): Pair<A | null, B | null> => ([fn(a) ? a : null, fn(b) ? b : null]),
  // filterBoth: <A,B>([a, b]: Pair<A,B>, fn: (_: A | B) => Boolean): (Pair<A, B> | null) => ((fn(a) && fn(b)) ? [a, b] : null),
  both: <A, B>([a, b]: Pair<A | null, B | null>): Pair<A, B> | null =>
    a !== null && b !== null ? [a, b] : null,
  first: <A, B>([a, _]: Pair<A, B>): A => a,
  second: <A, B>([_, b]: Pair<A, B>): B => b,
  unwrap: <A, B, T>([a, b]: Pair<A, B>, fn: (_a: A, _b: B) => T): T => fn(a, b),
};

export const Bread = (props: BreadProps) => {
  const [state, setState] = createStore<Recipe>(defaultRecipe());

  createEffect(() => props.title?.(state.total.name));

  const [getFocus, setFocus] = createSignal<HTMLElement | null>();
  createEffect(() => getFocus()?.focus());

  /// AHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH
  const [solved, setSolved] = createStore<Recipe>(deepcopy(unwrap(state)));

  batch(() => {
    solveRecipe(unwrap(state), setSolved);
  })

  let lastItem: HTMLElement | undefined;
  let lastMix: HTMLInputElement | undefined;

  const [undo, setUndo] = createStore<{ undo: string[]; redo: string[] }>({
    undo: [],
    redo: [],
  });

  const selection = createSelection<Item>();

  const selected = createMemo(() => selection.selected(state.items));
  const selectedInMix = ({ amounts }: Mix) =>
    selection.selected(state.items.filter((_, index) => amounts[index]));
  const selectedNotInMix = ({ amounts }: Mix) =>
    selection.selected(
      state.items.filter((_, index) => null === amounts[index])
    );

  const [getIsMoveMode, setIsMoveMode] = createSignal(false);
  createEffect(() => selected().length === 0 && setIsMoveMode(false));

  const [getShowHelp, setShowHelp] = createSignal<boolean>(false);

  /* this is instead of a copy for undo and redo */
  const saveState = () => JSON.stringify(state);
  const loadState = (s: string) => JSON.parse(s);

  const shareLink = () => {
    let url = new URL(window.location.href);
    url.hash = wasm.compact_base64(serializeToString(state));
    return url.href;
  };

  createEffect(() => {
    let hash, string, recipe: Recipe;

    if (!(hash = props.history().url.hash.slice(1))) return;

    if (   !(string = wasm.base64_expand(hash))
        || !(recipe = deserializeFromString(string)))
      return console.error("failed to load state from hash", { hash, string });

    /* maybe send this this through update() instead? */
    batch(() => {
      setState(reconcile(recipe));
      solveRecipe(recipe, setSolved);
    })
  });

  function solveRecipe(recipe: Recipe, update: (..._: any[]) => void) {
    type Path = (keyof Recipe | keyof Mix | keyof Amounts | number)

    let path, solutions;
    const paths = new Map();
    const writer = new wasm.RecipeWriter(4096)

    writeRecipe(recipe);

    if (!(solutions = writer.solve()))
      return console.error("failed solve");

    const newrecipe = deepcopy(recipe);

    /* setrattr and reconcile turns out to be much faster than the store
    * setter, even in batch */
    solutions.forEach((v, i) => (path = paths.get(i)) && setrattr(newrecipe, path, v))

    update(reconcile(newrecipe))

    return;

    function writeRecipe(recipe: Recipe) {
      let path: Path[], mix: Mix;

      path = ['total'];
      mix = recipe.total;

      writer.total();
      _writeAmounts(mix.total, ...path, 'total');

      writer.flour();
      _writeAmounts(mix.flour, ...path, 'flour');

      mix.amounts.forEach((amounts, i) => {
        const { is_flour } = recipe.items[i];

        const flags = new wasm.NewItemFlags();
        is_flour ? flags.flour() : flags.nonflour();
        amounts ? flags.total_item() : flags.hole();
        writer.new_item(flags);
        amounts && _writeAmounts(amounts, ...path, 'amounts', i)

        writeValue(writer.weight_less_mixes(), recipe.final[i], 'final', i)
      })

      recipe.mixes
        .forEach((mix, i) => {
          path = ['mixes', i];

          writer.new_mix();

          writer.total();
          _writeMixAmounts(mix.total, ...path, 'total');

          writer.flour();
          _writeMixAmounts(mix.flour, ...path, 'flour');

          mix.amounts.forEach((amounts, i) => {
            const { is_flour } = recipe.items[i];

            const flags = new wasm.NewItemFlags();
            is_flour ? flags.flour() : flags.nonflour();
            amounts ? flags.mix_item() : flags.hole();
            writer.new_item(flags);

            amounts && _writeMixAmounts(amounts, ...path, 'amounts', i)
          })
        })
    }

    function _writeAmounts(amounts: Amounts, ...path: Path[]) {
      writeValue(writer.weight(), amounts.weight, ...path, 'weight')
      writeValue(writer.bakers(), amounts.bakers, ...path, 'bakers')
      writeValue(writer.percent_in_mixes(), amounts.in_other, ...path, 'in_other')
    }

    function _writeMixAmounts(amounts: Amounts, ...path: Path[]) {
      writeValue(writer.weight(), amounts.weight, ...path, 'weight')
      writeValue(writer.bakers(), amounts.bakers, ...path, 'bakers')
      writeValue(writer.percent_of_total(), amounts.in_other, ...path, 'in_other')
    }

    function writeValue(value_index: number, input_value: number | null, ...path: Path[]) {
      paths.set(value_index, path);
      if (input_value !== null)
        writer.set(value_index, input_value)
    }

  }

  function update(update: Update) {
    console.log(update);

    const start = performance.now();

    // applyUpdate in batch breaks setFocus  FIXME
    applyUpdate(update);
    console.log("update", performance.now() - start, "ms");

    // let i = 2500;
    // while (i--)
      batch(() => solveRecipe(unwrap(state), setSolved));
    console.log("solve", performance.now() - start, "ms");
  }

  function applyUpdate(update: Update) {
    if (update === "undo") {
      const next = last(undo.undo);
      setUndo("undo", dropLast);
      setUndo("redo", appends(saveState()));
      setState(reconcile(loadState(next)));
      return;
    }

    if (update === "redo") {
      const next = last(undo.redo);
      setUndo("redo", dropLast);
      setUndo("undo", appends(saveState()));
      setState(reconcile(loadState(next)));
      return;
    }

    setUndo("undo", (a) => [...a.slice(-UNDO_LENGTH), saveState()]);
    setUndo("redo", []);

    if (update === "new-item") {
      const item: Item = { name: "", is_flour: last(state.items)?.is_flour };
      setState("items", appends(item));
      setState("total", "amounts", appends(Amounts()));
      setState("mixes", {}, "amounts", appends(null));
      setState("final", appends(0));
      setFocus(lastItem?.querySelector("input"));
      return;
    }

    if (update === "new-mix") {
      const amounts = state.items.map((i) =>
        selection.take(i) ? Amounts() : null
      );
      const mix: Mix = { ...Mix(), amounts };
      setState("mixes", appends(mix));
      setFocus(lastMix);
      return;
    }

    if (update === "remove-recipe-items") {
      const taken = state.items
        .map((item, index) => (selection.take(item) ? index : []))
        .flat();
      const removes = removesAt(...taken);
      setState("items", removes);
      setState("total", "amounts", removes);
      setState("mixes", {}, "amounts", removes);
      return;
    }

    if ("remove-mix" in update) {
      const { "remove-mix": index } = update;
      setState("mixes", removesAt(index));
      return;
    }

    if ("add-mix-items" in update) {
      const { "add-mix-items": index } = update;
      const { amounts } = state.mixes[index];
      const taken = state.items
        .map((item, index) =>
          amounts[index] === null && selection.take(item) ? index : []
        )
        .flat();
      setState("mixes", index, "amounts", taken, Amounts);
      return;
    }

    if ("remove-mix-items" in update) {
      const { "remove-mix-items": index } = update;
      const { amounts } = state.mixes[index];
      const taken = state.items
        .map((item, index) =>
          amounts[index] !== null && selection.take(item) ? index : []
        )
        .flat();
      setState("mixes", index, "amounts", taken, null);
      return;
    }

    if ("update-mix-name" in update) {
      const { "update-mix-name": name, mix } = update;
      if (mix === "total") {
        setState("total", "name", name);
      } else {
        setState("mixes", mix, "name", name);
      }
      return;
    }

    if ("update-final" in update) {
        const { "update-final": index, weight } = update;
        setState("final", index, weight)
        return;
    }

    if ("update-mix" in update) {
      if ("move-to" in update["update-mix"]) {
        const {
          "update-mix": { "move-to": moveTo },
        } = update;

        const countBefore = selection.count(state.items.slice(0, moveTo));
        const indexes = state.items
          .map((item, index) => (selection.take(item) ? index : []))
          .flat();
        const moves = <T,>(array: T[]) => {
          const moving = getsAt(...indexes)(array);
          const removed = removesAt(...indexes)(array);
          /* When the selection has an item _before_ the move destination,
           * then insert items just after destination. Otherwise, insert
           * just before the destination. */
          return insertsAt(countBefore > 0 ? moveTo + 1 - countBefore : moveTo)(
            moving
          )(removed);
        };

        setState("items", moves);
        setState("total", "amounts", moves);
        setState("mixes", {}, "amounts", moves);
        return;
      }

      const {
        "update-mix": { update: inner, item },
        mix,
      } = update;

      const [[attr, value]] = Object.entries(inner);

      switch (attr) {
        /* item */
        case "name":
        case "is_flour":
          if (item === "flour" || item === "total") throw item;
          else setState("items", item, attr, value);
          return;

        /* amounts */
        case "weight":
        case "bakers":
        case "in_other":
          if (item === "flour" || item === "total") {
            if (mix === "total") {
              setState("total", item, attr, value);
            } else {
              setState("mixes", mix, item, attr, value);
            }
          } else if (mix === "total") {
            setState("total", "amounts", item, attr, value);
          } else {
            setState("mixes", mix, "amounts", item, attr, value);
          }
          return;

        default:
          // donotwant(attr)
          throw attr;
      }
    }

    donotwant(update);
  }

  return (
    <>
      <article class="bread" classList={{ "show-help": getShowHelp() }} >

        <section class="leading-buttons">
          <p><a href={shareLink()}><Icon.Link /> Link to this recipe</a></p>
          <button onclick={() => setShowHelp(!getShowHelp())} accessKey="?">
            <Icon.Info /> {getShowHelp() ? "Hide help" : "Show help" }
          </button>

          <Show when={getShowHelp()}>
            <aside class="help">
              <p><Icon.Info /> The link above will take you back to this recipe. Share it or save it to see this recipe later.</p>
              <p><em>Here are some links to example recipes.</em></p>
              <ul>
                <li><a href="#BQIAAPYceyJpdGVtcyI6W3sibmFtZSI6IndoaXRlIiwiaXNfZmxvdXIiOnRydWV9LCEAmWF0ZXIg8J-avyYASGZhbHMnAJ9zYWx0IPCfp4ImAApfeWVhc3QiAAClXSwidG90YWwiOngAoFNhdHVyZGF5IFeiAPIZIEJyZWFkIChieSBLZW4gRm9ya2lzaCBpbiBGV1NZKSIsImFtb3VudNwA9hd3ZWlnaHQiOjEwMDAsImJha2VycyI6MCwiaW5fb3RoZXIiOjB9LCgACCUAPy43MigAEz8wMjEpABQqMDQpADNdLCJsAQ-DAAMKLgAHGwEE2AAIsAAKLQBwfSwibWl4ZQcBAGYA8AVpbmFsIjpbMCwwLDAsMCwwLDBdfQ">Saturday White Bread (by Ken Forkish in FWSY)</a></li>
                <li><a href="#GQMAAPIVeyJpdGVtcyI6W3sibmFtZSI6IndoaXRlIGZsb3VyIiwiaXNfCwB3OnRydWV9LCcAn29sZSB3aGVhdC0AEJlhdGVyIPCfmr9TAEhmYWxzVACfc2FsdCDwn6eCJgAApV0sInRvdGFsIjqDAPIqT3Zlcm5pZ2h0IENvdW50cnkgQnJvd24gKGJ5IEtlbiBGb3JraXNoIGluIEZXU1kpIiwiYW1vdW508AD_FndlaWdodCI6MCwiYmFrZXJzIjowLjcsImluX290aGVyIjowfSwnAAMPJQASPy43OCgAE0kwMjIwAQAaMjgAMl0sIo8BHzq1AAMKLgAHKAEP4gABCi0AeX0sIm1peGUBAm9sZXZhaW4tARAKSwAPKwEEGy7NAA8nAAUMLAFPbnVsbPkAGj8uMTL8AAE3MjE24AEKswAQfWEAIGluVQIzWzAsAgBgLDAsMF19">Overnight Country Brown (by Ken Forkish in FWSY)</a></li>
                <li><a href="#WgcAAPIVeyJpdGVtcyI6W3sibmFtZSI6IndoaXRlIGZsb3VyIiwiaXNfCwB3OnRydWV9LCcAn29sZSB3aGVhdC0ADz9yeWVMAAqZYXRlciDwn5q_JgBIZmFsc3MAn3NhbHQg8J-ngiYACl95ZWFzdCIACnJjcmFja2VkvQAPKgAKcXNvYWtlciCgAA8pAAzPdXJkb3VnaCBzZWVkKwAApV0sInRvdGFsIjpCAXVDcmF6eSBTOgBDdy8gQ5oAEVdXARJTfgAwJiBZ0wDAZWQgUHJlZmVybWVu4ABiYW1vdW50uwHAd2VpZ2h0IjowLCJisgDwBHMiOjAuNywiaW5fb3RoZXIiOjC7AQ8nAAIfMicAEg8lABIvLjdNABMvLjAoABUfMCoAFR8xoQATDycAFCswMT0BMl0sIhIDHzpGAQMLfgAG7AEEcwFIMTUwMJ4BCjEAcn0sIm1peGXNAQOIAwJ8AgnpAV9udWxsLAUABgZtAA9pAAUfMQUCBQiOACExfWMAD_IALg-FAAUA7gAF5QBxbGlxdWlkILQDi2VkIHN0YXJ09QAP1wAPMDAuM1MABGsBD3kABj8uMDUoACxIMS4wNSwBEjAsAQ99AAQ8LjAwcAIKvQEPZAFTB5wED18BJA84AhI_MC4zXwEUD64BBj8uNTZeAQEKNQEPbQEFC2sBD1wBSQBcACBpbs0FPVswLAIAYCwwLDBdfQ">Crazy Sourdough w/ Cracked Wheat Soaker & Yeasted Preferment</a></li>
              </ul>
            </aside>
          </Show>
        </section>

        <section class="mix totals">
          <header>
            <h1>
              <Field
                value={state.total.name}
                fmt={FmtText}
                update={(u) => update({ "update-mix-name": u, mix: "total" })}
              />
            </h1>
          </header>

          <Show when={getShowHelp()}>
            <aside class="help">
              <p><Icon.Info /> This table lists the <em>total</em> amounts of each ingredient in the recipe.</p>
              <p><Icon.Flour /> is a flour ingredient, added to total flour</p>
              <p><Icon.NonFlour /> is non-flour ingredient, excluded from total flour</p>
              <p>The <em>baker's percentage</em> <Icon.One /> expresses the <em>weight</em> <Icon.Two /> as a percentage of the <em>total flour</em> in the table.</p>
              <p>Here, in the totals table, column <Icon.Three /> is the percentage of the total amount of each ingredient that comes from all mixes.</p>
              <p>In mix tables, it's the percentage of the total amount of each ingredient that comes from the mix.</p>
            </aside>
            <aside class="help">
              <p><Icon.Info /> For simple recipes, set the total weight <Icon.Two /> of the dough in the first row. Then <Icon.NewIngredient /> add ingredients and set the baker's percentages <Icon.One /> for each. From that information, it should calculate the weight of each item automatically.</p>
            </aside>
          </Show>

          <MixTable
            items={state.items}
            mix={Pair.of(state.total, solved.total)}
            update={(u) => update({ "update-mix": u, mix: "total" })}
            refItem={(r) => {
              /* this corresponds to the special ref attribute in solidjs;
               * newly added item elements are assigned to lastItem */
              lastItem = r;
            }}
            selection={selection}
            emptyText="click new ingredient"
            toolMode={(item) =>
              getIsMoveMode()
                ? selection.includes(item)
                  ? null
                  : "move"
                : "selection"
            }
            showHeader={getShowHelp()}
          />

          <footer>
            <div>
              <Show when={last(undo.undo)}>
                {
                  <button onclick={() => update("undo")}>
                    <Icon.Undo /> undo
                  </button>
                }
              </Show>
              <Show when={last(undo.redo)}>
                {
                  <button onclick={() => update("redo")}>
                    <Icon.Redo /> redo
                  </button>
                }
              </Show>
            </div>
            <Show when={nullIfEmpty(selected())} keyed>
              {(items) => (
                <button onclick={() => update("remove-recipe-items")}>
                  <Icon.DeleteIngredient /> remove <ItemNames items={items} />
                </button>
              )}
            </Show>
            <Show when={!selected().length} keyed>
              <button onclick={() => update("new-item")} accessKey="i">
                <Icon.NewIngredient /> new ingredient
              </button>
            </Show>
            <button onclick={() => update("new-mix")} accessKey="m">
              <Icon.NewTable /> new mix
            </button>
            <Toggle
              class="toggle-move-mode"
              value={getIsMoveMode()}
              title="select items to reorder them"
              update={(v) => setIsMoveMode(v)}
              readonly={!selected().length}
            >
              <Icon.Move />
            </Toggle>
          </footer>
        </section>

        <For each={state.mixes}>
          {(mix, i) => (
            <section class="mix">
              <header>
                <h2>
                  <Field
                    placeholder="name this mix?"
                    value={mix.name}
                    update={(u) => update({ "update-mix-name": u, mix: i() })}
                    fmt={FmtText}
                    ref={lastMix}
                  />
                </h2>
              </header>

              <Show when={getShowHelp()}>
                <aside class="help">
                  <p><Icon.Info /> This is a mix table.</p>
                  <p>A mix is like a soaker or a preferment or a starter.</p>
                </aside>
              </Show>

              <MixTable
                items={state.items}
                mix={Pair.of(mix, coalesce(solved.mixes[i()], Mix))}
                update={(u) => update({ "update-mix": u, mix: i() })}
                refItem={(r) => {
                  lastItem = r;
                }}
                toolMode={() => "selection"}
                selection={selection}
                emptyText="select items to add them to this mix"
              />

              <footer class="tool">
                <Show when={nullIfEmpty(selectedInMix(mix))} keyed>
                  {(items) => (
                    <button onclick={() => update({ "remove-mix-items": i() })}>
                      <Icon.RemoveIngredient /> remove{" "}
                      <ItemNames items={items} />
                    </button>
                  )}
                </Show>
                <Show when={nullIfEmpty(selectedNotInMix(mix))} keyed>
                  {(items) => (
                    <button onclick={() => update({ "add-mix-items": i() })}>
                      <Icon.AddIngredient /> add <ItemNames items={items} />
                    </button>
                  )}
                </Show>
                <button onclick={() => update({ "remove-mix": i() })}>
                  <Icon.DeleteTable /> remove mix
                </button>
              </footer>
            </section>
          )}
        </For>
        {/* .mix and .mix-item classes used here for Ui.Focus.rowKbNav to work,
          * could be improved to be more explicit... */}
        <section class="mix">
          <header>
            <h2>
              <Field value={"Final"} fmt={FmtText} readonly />
            </h2>
          </header>

          <Show when={getShowHelp()}>
            <aside class="help">
              <p><Icon.Info /> This lists the items for the final dough; each mix and the amount of each ingredient not used in any mix.</p>
              <p>Add the weights here to reach the ingredient totals for the recipe.</p>
            </aside>
          </Show>

          <div>
            <For each={state.items}>
              {(item, i) => (
                <div class="mix-item" onkeydown={rowKbNav}>
                  <Field
                    class="title"
                    placeholder="untitled"
                    value={item.name}
                    fmt={FmtText}
                    readonly
                  />
                  <Show when={i() in state.final} fallback={"???"}>
                    <PairField
                      class="weight"
                      fmt={FmtNull(FmtWeight)}
                      value={Pair.of(
                        state.final[i()],
                        coalesce(solved.final[i()], () => 0)
                      )}
                      update={(u) => update({ "update-final": i(), weight: u })}
                    />
                  </Show>
                </div>
              )}
            </For>
            <For each={state.mixes}>
              {(mix, i) => (
                <div class="mix-item" onkeydown={rowKbNav}>
                  <Field
                    class="title"
                    placeholder="untitled"
                    value={mix.name}
                    fmt={FmtText}
                    readonly
                  />
                  <Show when={i() in state.final} fallback={"???"}>
                    <PairField
                      class="weight"
                      fmt={FmtNull(FmtWeight)}
                      value={Pair.of(
                        state.mixes[i()].total.weight,
                        coalesce(solved.mixes[i()]?.total.weight, () => 0)
                      )}
                      readonly
                    />
                  </Show>
                </div>
              )}
            </For>
          </div>

        </section>
      </article>
    </>
  );
};

type ToolMode = "selection" | "move" | null;

type MixTableProps = {
  items: Item[];
  mix: Pair<Mix>;
  update: (_: MixTableUpdate) => void;
  refItem?: (_: HTMLElement) => void;
  emptyText: string;
  toolMode: (_: Item) => ToolMode;
  selection: Selection<Item>;
  showHeader?: boolean;
} & JSX.HTMLAttributes<HTMLDivElement>;

type MixTableUpdate =
  | { item: number | "total" | "flour"; update: RowUpdate }
  | { "move-to": number };

function MixTable(props: MixTableProps) {
  const [self, rest] = splitProps(props, [
    "items",
    "mix",
    "update",
    "refItem",
    "emptyText",
    "toolMode",
    "selection",
    "showHeader",
  ]);

  return (
    <div class="mix-item-list" {...rest}>
      <Show when={self.showHeader}><MixTableHeader /></Show>
      <Show when={Pair.first(self.mix).amounts.some(Boolean)}>
        <ItemRow
          class="total"
          item={TOTAL}
          amounts={Pair.map(self.mix, (mix) => mix.total)}
          update={(update) => self.update({ item: "total", update })}
        />
        <ItemRow
          class="total"
          item={TOTAL_FLOUR}
          amounts={Pair.map(self.mix, (mix) => mix.flour)}
          update={(update) => self.update({ item: "flour", update })}
        />
      </Show>
      <For each={self.items}>
        {(item, i) => (
          /* we can't use keyed here because the Pair/tuple instance thing is
           * new each time -- and we can't key it on an expression? */
          <Show when={Pair.first(self.mix).amounts[i()]} keyed>
            <ItemRow
              classList={{ "is-selected": self.selection?.includes(item) }}
              item={item}
              amounts={Pair.map(self.mix, (mix) =>
                coalesce(mix.amounts[i()], Amounts)
              )}
              update={(update) => self.update({ item: i(), update })}
              tool={
                <>
                  <Show when={self.toolMode(item) === "selection"}>
                    <Toggle
                      value={self.selection.includes(item)}
                      onclick={() => self.selection.toggle(item)}
                    >
                      {self.selection.includes(item) ? (
                        <span class="icon">
                          <Icon.SquareFill />
                        </span>
                      ) : (
                        <span class="icon">
                          <Icon.Square />
                        </span>
                      )}
                    </Toggle>
                  </Show>
                  <Show when={self.toolMode(item) === "move"}>
                    <button onclick={() => self.update({ "move-to": i() })}>
                      <Icon.Move />
                    </button>
                  </Show>
                </>
              }
              ref={self.refItem}
            />
          </Show>
        )}
      </For>
      <Show when={!Pair.first(self.mix).amounts.some(Boolean)}>
        <tr>
          <td colspan="5">
            <p class="muted">{self.emptyText}</p>
          </td>
        </tr>
      </Show>
    </div>
  );
}

function MixTableHeader() {
  return (
    <div class="mix-item help-header">
      <div class="title" />
      <div class="is-flour" />
      <div class="bakers"><Icon.One /></div>
      <div class="weight"><Icon.Two /></div>
      <div class="in-other"><Icon.Three /></div>
      <div class="tool" />
    </div>
  )
}

type RowUpdate =
  | { name: string }
  | { is_flour: boolean }
  | { bakers: number | null }
  | { in_other: number | null }
  | { weight: number | null };

type ItemRowProps = {
  item: Item;
  amounts: Pair<Amounts>;
  update: (_: RowUpdate) => void;
  tool?: JSX.Element;
} & JSX.HTMLAttributes<HTMLDivElement>;

function ItemRow(props: ItemRowProps) {
  const [self, rest] = splitProps(props, [
    "item",
    "amounts",
    "update",
    "tool",
    "class",
  ]);

  const isFixed =
    Object.is(self.item, TOTAL) || Object.is(self.item, TOTAL_FLOUR);

  return (
    /* nasty class mixing */
    <div class={`mix-item ${self.class || ""}`} onkeydown={rowKbNav} {...rest}>
      <Field
        class="title"
        placeholder="type ingredient name"
        value={self.item.name}
        update={(name) => self.update({ name })}
        readonly={isFixed}
        fmt={FmtText}
      />
      <FlourToggle
        class="is-flour"
        value={self.item.is_flour}
        update={(is_flour) => self.update({ is_flour })}
        readonly={isFixed}
      />
      <PairField
        class="percent bakers"
        fmt={FmtNull(FmtPercent)}
        value={Pair.map(self.amounts, (a) => a.bakers)}
        update={(bakers) => self.update({ bakers })}
      />
      <PairField
        class="weight"
        fmt={FmtNull(FmtWeight)}
        value={Pair.map(self.amounts, (a) => a.weight)}
        update={(weight) => self.update({ weight })}
      />
      <PairField
        class="percent in-other"
        fmt={FmtNull(FmtPercent)}
        value={Pair.map(self.amounts, (a) => a.in_other)}
        update={(in_other) => self.update({ in_other })}
      />
      <div class="tool">{self.tool}</div>
    </div>
  );
}

function ItemNames(props: { items: Item[] }) {
  if (props.items.length <= 2) {
    return (
      <For each={props.items}>
        {(item, i) => (
          <>
            <Show when={i() > 0}> & </Show>
            <ItemName name={item.name} />
          </>
        )}
      </For>
    );
  } else {
    return <b>{props.items.length} items</b>;
  }
}

const ItemName = (props: { name: string }) =>
  props.name ? (
    <span class="item-name">{props.name}</span>
  ) : (
    <span class="item-name untitled">untitled</span>
  );

/* Field */

type Fmt<T> = {
  parse: (_: string) => T;
  display: (_: T) => string;
} & JSX.InputHTMLAttributes<HTMLInputElement>;

const FmtText: Fmt<string> = {
  parse: (_) => _,
  display: (_) => _,
  inputmode: "text",
  maxlength: "128",
};

const FmtPercent: Fmt<number> = {
  parse: parsePercent,
  display: percent,
  inputmode: "decimal",
  maxlength: "8",
};

const FmtWeight: Fmt<number> = {
  parse: parseWeightToGrams,
  display: grams,
  inputmode: "decimal",
  maxlength: "12",
};

const FmtNull = <T,>({ parse, display, ...fmt }: Fmt<T>): Fmt<T | null> => ({
      parse: (s: string) => s === "" ? null : parse(s),
      display: (v: T | null) => v === null ? "" : display(v),
      ...fmt,
    })

type FieldProps<T> = {
  value: T;
  update?: (_: T) => void;
  fmt: Fmt<T>;
} & Omit<JSX.InputHTMLAttributes<HTMLInputElement>, "value">;

function Field<T>(props: FieldProps<T>) {
  const [self, rest_] = splitProps(props, ["value", "update", "fmt"]);
  const [shared, rest] = splitProps(rest_, ["class", "classList"]);
  const { parse, display, ...fmt_rest } = self.fmt;

  return (
    <input
      size="14"
      type="text"
      placeholder="--" // ∅ not sure which to use here
      {...fmt_rest}
      {...rest}
      class={shared.class}
      classList={{ "is-zero": isZero(self.value), ...shared.classList }}
      value={self.value === null ? "" : display(self.value)}
      onchange={(e) => {
        /* parse("0") and parse("00") evaluates to the same thing, if done
         * in sequence, the second value will not cause the input's
         * displayed value to update, leaving it showing "00" even though
         * display(0) evaluates to "0"
         *
         * todo there ought to be a "reactive" way to do this? */
        self.update?.(parse(e.currentTarget.value));
        e.currentTarget.value = display(self.value);
      }}
    />
  );
}

type PairFieldProps<T> = {
  value: Pair<T>;
} & Omit<FieldProps<T>, "value">;

function PairField<T>(props: PairFieldProps<T>) {
  const [self, rest] = splitProps(props, ["value"]);

  return (
    <Field
      classList={{ "from-user": isNonNull(Pair.first(self.value)) }}
      value={Pair.unwrap(self.value, (a, b) => (isNonNull(a) ? a : b))}
      {...rest}
    />
  );
}

/* Toggle */

type ToggleProps = {
  value: boolean;
  update?: (_: boolean) => void;
  readonly?: boolean;
  children?: JSX.Element;
} & JSX.HTMLAttributes<HTMLButtonElement>;

function Toggle(props: ToggleProps) {
  const [self, rest] = splitProps(props, [
    "value",
    "update",
    "readonly",
    "children",
  ]);
  return (
    <button
      classList={{
        "is-checked": self.value == true,
      }}
      disabled={self.readonly}
      aria-checked={self.value}
      onclick={() => self.update?.(!self.value)}
      {...rest}
    >
      {self.children ?? <EmSpace />}
    </button>
  );
}

function FlourToggle(props: ToggleProps) {
  return (
    <Toggle {...props}>
      {props.value ? <Icon.Flour /> : <Icon.NonFlour />}
    </Toggle>
  );
}

const EmSpace = () => <>&emsp;</>;
