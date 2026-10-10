import { useEffect, useState } from "react";
import { fetchReleases, type ReleaseInfo } from "./release";

export type ReleasesState =
  | { status: "loading" }
  | { status: "ready"; releases: ReleaseInfo[] }
  | { status: "error" };

export function useReleases(): ReleasesState {
  const [state, setState] = useState<ReleasesState>({ status: "loading" });
  useEffect(() => {
    let alive = true;
    let storage: Storage | undefined;
    try {
      storage = window.sessionStorage;
    } catch {
      storage = undefined;
    }
    fetchReleases(fetch, storage)
      .then((releases) => alive && setState(releases.length ? { status: "ready", releases } : { status: "error" }))
      .catch(() => alive && setState({ status: "error" }));
    return () => {
      alive = false;
    };
  }, []);
  return state;
}
