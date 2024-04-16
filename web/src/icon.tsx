import { JSX } from "solid-js"

import SvgCircle from "~icons/ph/circle";
import SvgCheckCircle from "~icons/ph/check-circle";
import SvgMinusCircleBold from "~icons/ph/minus-circle-bold";
import SvgXCircle from "~icons/ph/x-circle";
import SvgXCircleBold from "~icons/ph/x-circle-bold";
import SvgPlusCircle from "~icons/ph/plus-circle";
import SvgPlusCircleBold from "~icons/ph/plus-circle-bold";
import SvgSwap from "~icons/ph/swap";

import SvgSquare from "~icons/ph/square";
import SvgSquareFill from "~icons/ph/square-fill";
import SvgRectangle from "~icons/ph/rectangle";
import SvgSquareSplitVertical from "~icons/ph/square-split-vertical"
import SvgHardDrive from "~icons/ph/hard-drive"
import SvgList from "~icons/ph/list"

import SvgTable from "~icons/ph/table";
import SvgStack from "~icons/ph/stack";
import SvgGrains from "~icons/ph/grains";
import SvgGrainsSlash from "~icons/ph/grains-slash";
import SvgLinkSimple from "~icons/ph/link-simple";
import SvgInfo from "~icons/ph/info";
import SvgX from "~icons/ph/x";
import SvgArrowArcLeft from "~icons/ph/arrow-arc-left";
import SvgArrowArcRight from "~icons/ph/arrow-arc-right";
import SvgTrash from "~icons/ph/trash";
import SvgTrashBold from "~icons/ph/trash-bold";
import SvgSparkle from "~icons/ph/sparkle";
import SvgSparkleBold from "~icons/ph/sparkle-bold";

import SvgNumberCircleOne from "~icons/ph/number-circle-one";
import SvgNumberCircleTwo from "~icons/ph/number-circle-two";
import SvgNumberCircleThree from "~icons/ph/number-circle-three";

const Span = (props: {children?: JSX.Element, title?: string}) => <span class="icon" title={props.title}>{props.children}</span>

export const CheckCircle = () => <Span><SvgCheckCircle/></Span>;
export const Circle = () => <Span><SvgCircle/></Span>;
export const Flour = () => <Span><SvgGrains/></Span>;
export const Link = () => <Span><SvgLinkSimple/></Span>;
export const Info = () => <Span><SvgInfo/></Span>;
export const X = () => <Span><SvgX/></Span>;
export const NonFlour = () => <Span><SvgGrainsSlash/></Span>;
export const PlusCircle = () => <Span><SvgPlusCircle/></Span>;
export const Square = () => <Span><SvgSquare/></Span>;
export const SquareFill = () => <Span><SvgSquareFill/></Span>;
export const Undo = () => <Span><SvgArrowArcLeft/></Span>;
export const Redo = () => <Span><SvgArrowArcRight/></Span>;
export const Share = () => <Span><SvgLinkSimple/></Span>;
export const Move = () => <Span><SvgSwap/></Span>;

const Ingredient = SvgHardDrive;
// const Ingredient = SvgList;
const Table = SvgStack;

export const NewIngredient = () => <Span><Ingredient/><SvgSparkleBold/></Span>;
export const NewTable = () => <Span><Table/><SvgSparkleBold/></Span>;
export const DeleteIngredient = () => <Span><Ingredient/><SvgTrashBold/></Span>;
export const DeleteTable = () => <Span><Table/><SvgTrashBold/></Span>;
export const AddIngredient = () => <Span><Ingredient/><SvgPlusCircleBold/></Span>;
export const RemoveIngredient = () => <Span><Ingredient/><SvgMinusCircleBold/></Span>;

export const One = () => <Span title="one"><SvgNumberCircleOne /></Span>
export const Two = () => <Span title="two"><SvgNumberCircleTwo /></Span>
export const Three = () => <Span title="three"><SvgNumberCircleThree /></Span>
