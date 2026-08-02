import type { InjectionKey } from "vue";

export type WorkspaceFileEntry = {
  path: string;
  text: string;
};

export type WorkspaceSlotPayload = {
  slot: string;
  files: WorkspaceFileEntry[];
  linked_source?: boolean;
  origin_root?: string;
  origin_source?: string;
  display_root?: string;
  display_source?: string;
  refreshed_from_source?: boolean;
};

export type ChooseSourceResult = {
  path?: string;
  error?: string;
  cancelled?: boolean;
};

export type SaveWorkspaceAsResult = {
  destination?: string;
  linked_source?: boolean;
  origin_root?: string;
  origin_source?: string;
  display_root?: string;
  display_source?: string;
  error?: string;
};

export type WorkspaceChoice = {
  slot: string;
  label: string;
  origin_source?: string;
};

export type LinkSourceResult = {
  slot?: string;
  label?: string;
  origin_source?: string;
  error?: string;
};

export type ClearWorkspaceSlotsResult = {
  deleted?: string[];
  choices?: WorkspaceChoice[];
  error?: string;
};

export type RunebenderStatePayload = {
  nodeId: string;
  font: string;
  glyphData: string;
};

export type TraceBackgroundGlyphArgs = {
  slot: string;
  master: string;
  glyph: string;
  image: File;
  unicode?: string;
  width: number;
  targetHeight: number;
  xOffset: number;
  yOffset: number;
  imageWidth?: number;
  imageHeight?: number;
  designX?: number;
  designY?: number;
  designScaleX?: number;
  designScaleY?: number;
  grid?: number;
  accuracy?: number;
  smooth?: number;
  alphamax?: number;
  globalFit?: boolean;
  invert?: boolean;
  threshold?: number | null;
};

export type TraceBackgroundCandidateArgs = TraceBackgroundGlyphArgs & {
  candidateName?: string;
  unitsPerEm?: number;
  ascender?: number;
  descender?: number;
};

export type TraceBackgroundGlyphResult = {
  success?: boolean;
  glyph?: string;
  glif?: string;
  source_ufo?: string;
  command?: string[];
  error?: string;
};

export type TraceBackgroundCandidateResult = {
  success?: boolean;
  candidate_slot?: string;
  trace_request?: string;
  request_id?: string;
  glyph?: string;
  master?: string;
  report?: Record<string, unknown>;
  error?: string;
};

// A change made to the workspace by something OTHER than this editor —
// an AI agent editing the UFO on disk, a git checkout, another tool.
// `path` is slot-prefixed to match MasterData.glyphPaths values.
export type WorkspaceExternalChange = {
  type: "change" | "delete";
  path: string;
  text?: string;
};

export type RunebenderHost = {
  log?(level: string, message: string): void;
  publishState(payload: RunebenderStatePayload): Promise<void>;
  loadWorkspaceSlot(slot: string): Promise<WorkspaceSlotPayload | null>;
  listWorkspaceSlots(): Promise<WorkspaceChoice[]>;
  clearWorkspaceSlots(): Promise<ClearWorkspaceSlotsResult>;
  workspacePreviewUrl(slot: string, params: URLSearchParams): string;
  drawBotPresetSource(name: string): Promise<string | null>;
  writeWorkspaceFile(path: string, text: string): Promise<Response>;
  chooseSource(mode?: "source" | "folder"): Promise<ChooseSourceResult>;
  linkSource(args: {
    sourcePath: string;
    sourceKind: string;
    workspaceName: string;
  }): Promise<{ response: Response; data: LinkSourceResult }>;
  saveWorkspaceAs(args: {
    slot: string;
    destination: string;
    relink: boolean;
  }): Promise<{ response: Response; data: SaveWorkspaceAsResult }>;
  traceBackgroundGlyph(args: TraceBackgroundGlyphArgs): Promise<{
    response: Response;
    data: TraceBackgroundGlyphResult;
  }>;
  traceBackgroundCandidate(args: TraceBackgroundCandidateArgs): Promise<{
    response: Response;
    data: TraceBackgroundCandidateResult;
  }>;
  invalidateWorkspacePath(path: string): Promise<void>;
  // Optional: hosts that can open a local folder themselves (the File
  // System Access host) own the picker and the resulting workspace;
  // the editor's "Open UFO" affordances call these instead of running
  // its own picker, and the load is triggered by the host via the
  // fontPathRef it was wired to at boot.
  openWorkspaceFolder?(): Promise<{
    slot?: string;
    cancelled?: boolean;
    error?: string;
  }>;
  // Optional: file-first opening. pickSourceFile shows a FILE picker
  // (.designspace / .glyphs are clickable — the gesture designers
  // expect). A .glyphs is self-contained and comes back as a File; a
  // .designspace needs its sibling UFOs, so the host holds the picked
  // handle and the editor calls grantSourceFolder (from a fresh user
  // gesture) — that folder picker opens AT the file's own directory,
  // so it's one click on Select.
  pickSourceFile?(): Promise<{
    kind?: "glyphs" | "designspace";
    file?: File;
    name?: string;
    cancelled?: boolean;
    error?: string;
  }>;
  grantSourceFolder?(): Promise<{
    slot?: string;
    cancelled?: boolean;
    error?: string;
  }>;
  reopenStoredWorkspace?(): Promise<{
    slot?: string;
    cancelled?: boolean;
    error?: string;
  }>;
  // Name of the workspace remembered from a previous visit, if any —
  // drives the welcome panel's "Reopen <name>" button.
  storedWorkspaceName?(): string | null;
  // Optional: recently opened folders/.glyphs files, newest first —
  // drives the system menu's Open Recent section. openRecentWorkspace
  // must run inside a user gesture (permission re-request); folders
  // come back as { slot }, single files as { file }.
  listRecentWorkspaces?(): Promise<
    { index: number; name: string; kind: "folder" | "file" }[]
  >;
  openRecentWorkspace?(index: number): Promise<{
    slot?: string;
    file?: File;
    cancelled?: boolean;
    error?: string;
  }>;
  // Optional: hosts that can observe the workspace (the local server's
  // file watcher) call `handler` whenever files change externally. The
  // editor applies the changes live — the "watch the agent work" loop.
  // The handler returns the paths it actually APPLIED; changes it held
  // back (unsaved local edits) are excluded, so the host must keep its
  // conflict state (e.g. ETags) at the editor's version — a later save
  // of the held-back file then surfaces as a conflict instead of
  // silently overwriting the external edit.
  watchWorkspaceChanges?(
    handler: (
      changes: WorkspaceExternalChange[],
    ) => void | string[] | Promise<void | string[]>,
  ): void;
};

export const runebenderHostKey: InjectionKey<RunebenderHost> = Symbol("runebender-host");
