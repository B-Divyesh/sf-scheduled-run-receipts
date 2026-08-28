export type SlotState = 'success' | 'missing' | 'late' | 'failed' | 'running' | 'pending' | 'overlap';

export interface Slot {
  job: string;
  scheduled_at: string;
  state: SlotState;
  run_id?: string;
  started_at?: string;
  finished_at?: string;
}

export interface CheckReport {
  generated_at: string;
  since: string;
  healthy: boolean;
  counts: Record<string, number>;
  slots: Slot[];
}

const validStates = new Set<SlotState>(['success', 'missing', 'late', 'failed', 'running', 'pending', 'overlap']);

export function parseReport(value: unknown): CheckReport {
  if (!value || typeof value !== 'object') throw new Error('The file does not contain a check report.');
  const report = value as Partial<CheckReport>;
  if (!Array.isArray(report.slots)) throw new Error('The report is missing its slots list.');
  const slots = report.slots.map((slot, index) => {
    if (!slot || typeof slot !== 'object') throw new Error(`Slot ${index + 1} is invalid.`);
    const item = slot as Partial<Slot>;
    if (typeof item.job !== 'string' || !item.job) throw new Error(`Slot ${index + 1} has no job name.`);
    if (typeof item.scheduled_at !== 'string' || Number.isNaN(Date.parse(item.scheduled_at))) throw new Error(`Slot ${index + 1} has an invalid expected time.`);
    if (!validStates.has(item.state as SlotState)) throw new Error(`Slot ${index + 1} has an unknown state.`);
    return item as Slot;
  });
  return {
    generated_at: typeof report.generated_at === 'string' ? report.generated_at : new Date().toISOString(),
    since: typeof report.since === 'string' ? report.since : '',
    healthy: report.healthy === true,
    counts: report.counts && typeof report.counts === 'object' ? report.counts : {},
    slots,
  };
}

export function compactSlots(slots: Slot[], max = 14): Slot[] {
  return [...slots].sort((a, b) => Date.parse(b.scheduled_at) - Date.parse(a.scheduled_at)).slice(0, max).reverse();
}

export function stateLabel(state: SlotState): string {
  return ({ success: 'Verified', missing: 'Missing', late: 'Late', failed: 'Failed', running: 'Running', pending: 'Pending', overlap: 'Overlap' })[state];
}
