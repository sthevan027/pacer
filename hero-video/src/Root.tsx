import { Composition } from "remotion";
import { Hero, FPS, DURATION } from "./Hero";

export const Root = () => (
  <Composition id="Hero" component={Hero} durationInFrames={DURATION} fps={FPS} width={1920} height={1080} />
);
