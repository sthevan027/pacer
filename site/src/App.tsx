import { useState, type CSSProperties } from "react";
import { Header } from "./components/Header";
import { Hero } from "./components/Hero";
import { Features } from "./components/Features";
import { Showcase, ACCENTS } from "./components/Showcase";
import { Privacy } from "./components/Privacy";
import { Install } from "./components/Install";
import { Changelog } from "./components/Changelog";
import { Footer } from "./components/Footer";
import { useReleases } from "./lib/useReleases";

export default function App() {
  const [accent, setAccent] = useState<string>(ACCENTS[0].hex);
  const releases = useReleases();
  const list = releases.status === "ready" ? releases.releases : undefined;
  return (
    <div style={{ "--accent": accent } as CSSProperties}>
      <Header />
      <main>
        <Hero release={list?.[0]} />
        <Features />
        <Showcase accent={accent} onAccent={setAccent} />
        <Privacy />
        <Install />
        <Changelog releases={list} />
      </main>
      <Footer />
    </div>
  );
}
