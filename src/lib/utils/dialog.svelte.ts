/** Imperative confirm + notify API. Replaces native alert()/confirm()
 *  with in-app modals/toasts.
 *
 *  USAGE:
 *    if (!(await confirm({ title: 'Discard changes', message: '...', danger: true }))) return;
 *    notify('Saved', { kind: 'success' });
 *    notify(formatError(err), { kind: 'error', durationMs: 0 }); // sticky
 */

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  /** When true, the confirm button uses <Button variant="danger">. */
  danger?: boolean;
}

export interface NotifyOptions {
  kind?: 'info' | 'success' | 'error';
  /** Auto-dismiss after this many ms; 0 = sticky until clicked. */
  durationMs?: number;
}

export interface ConfirmRequest {
  id: number;
  opts: ConfirmOptions;
  resolve: (value: boolean) => void;
}

export interface ToastEntry {
  id: number;
  message: string;
  kind: 'info' | 'success' | 'error';
  durationMs: number;
}

let _nextId = 1;
function nextId(): number { return _nextId++; }

const _confirms = $state<{ list: ConfirmRequest[] }>({ list: [] });
const _toasts = $state<{ list: ToastEntry[] }>({ list: [] });

/** DialogHost reads these to render. */
export function _readState(): { confirms: ConfirmRequest[]; toasts: ToastEntry[] } {
  return { confirms: _confirms.list, toasts: _toasts.list };
}

export function confirm(opts: ConfirmOptions): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    _confirms.list = [..._confirms.list, { id: nextId(), opts, resolve }];
  });
}

export function notify(message: string, opts: NotifyOptions = {}): void {
  const id = nextId();
  const kind = opts.kind ?? 'info';
  const durationMs = opts.durationMs ?? 3000;
  _toasts.list = [..._toasts.list, { id, message, kind, durationMs }];
  if (durationMs !== 0) {
    setTimeout(() => {
      _toasts.list = _toasts.list.filter((t) => t.id !== id);
    }, durationMs);
  }
}

/** DialogHost calls this when the user picks an answer. */
export function _resolveConfirm(id: number, value: boolean): void {
  const req = _confirms.list.find((c) => c.id === id);
  if (req) {
    req.resolve(value);
    _confirms.list = _confirms.list.filter((c) => c.id !== id);
  }
}

export function _dismissToast(id: number): void {
  _toasts.list = _toasts.list.filter((t) => t.id !== id);
}
