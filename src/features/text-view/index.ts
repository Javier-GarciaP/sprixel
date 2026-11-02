export { TextView } from "./TextView";
export { coreTextApi, asGridFailure } from "./api";
export type {
  GridCheck,
  GridError,
  GridFailure,
  GridPreview,
  GridSummary,
  SpriteRef,
  TextApi,
} from "./types";
// A stand-in for the core, for the tests of the screens that show this view.
export { FakeTextApi } from "./testing";
