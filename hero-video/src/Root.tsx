import { Composition } from "remotion";
import { Hero, FPS, DURATION } from "./Hero";
import { Social, SOCIAL_DURATION } from "./Social";

export const Root = () => (
  <>
    <Composition id="Hero" component={Hero} durationInFrames={DURATION} fps={FPS} width={1920} height={1080} />
    <Composition id="Social" component={Social} durationInFrames={SOCIAL_DURATION} fps={FPS} width={1080} height={1920} />
  </>
);
