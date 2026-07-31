// IndexedDB persistence for the picked font directory handle.
//
// FileSystemDirectoryHandle is structured-cloneable, so storing it here
// lets a return visit reopen the same folder without re-picking. This
// is also the prerequisite for Chrome's persistent-permission prompt
// ("Allow on every visit"): that three-way prompt only appears when a
// STORED handle re-requests permission — a fresh showDirectoryPicker()
// never offers it.

const DB_NAME = "runebender-fsaccess";
const STORE = "handles";
const KEY = "last-workspace";

type StoredWorkspace = {
  handle: FileSystemDirectoryHandle;
  name: string;
};

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(STORE);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function withStore<T>(
  mode: IDBTransactionMode,
  fn: (store: IDBObjectStore) => IDBRequest<T>,
): Promise<T> {
  const db = await openDb();
  try {
    return await new Promise<T>((resolve, reject) => {
      const req = fn(db.transaction(STORE, mode).objectStore(STORE));
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
  } finally {
    db.close();
  }
}

export async function saveWorkspaceHandle(
  handle: FileSystemDirectoryHandle,
): Promise<void> {
  const value: StoredWorkspace = { handle, name: handle.name };
  await withStore("readwrite", (store) => store.put(value, KEY));
}

export async function loadWorkspaceHandle(): Promise<StoredWorkspace | null> {
  try {
    const value = await withStore<StoredWorkspace | undefined>(
      "readonly",
      (store) => store.get(KEY),
    );
    return value?.handle ? value : null;
  } catch {
    return null;
  }
}

export async function clearWorkspaceHandle(): Promise<void> {
  try {
    await withStore("readwrite", (store) => store.delete(KEY));
  } catch {
    // Nothing to clear.
  }
}
