export { App } from './App.tsx';
export { attachHost, host, type ArtifactKind, type Autostart, type HostAdapter, type Lifecycle } from './host.tsx';
export {
  isConsole,
  useConsole,
  type ConsoleSession,
  type InstallOutcome,
  type KnownDevice,
  type OtaOutcome,
} from './console.ts';
export { seed } from './stores/session.ts';
export { PATHS, SECTIONS, sectionFor } from './routes.ts';

// the chrome a host surface needs to render its own rows alongside the screens
export { Progress } from './components/Progress.tsx';
export { BackButton, ErrorNote, Hint, Screen, Section } from './components/Screen.tsx';
export { Icon, type IconName } from './lib/icons.tsx';
export { basename, bytes, clock, day, peerHost, rate, since } from './lib/format.ts';
