import { describe, expect, it } from 'vitest';
import { compactSlots, parseReport, stateLabel } from './receipt';

describe('check report viewer', () => {
  it('validates and sorts CLI output', () => {
    const report = parseReport({ generated_at: '2026-08-28T00:00:00Z', since: '', healthy: false, counts: {}, slots: [
      { job: 'backup', scheduled_at: '2026-08-28T02:00:00Z', state: 'missing' },
      { job: 'backup', scheduled_at: '2026-08-27T02:00:00Z', state: 'success' },
    ] });
    expect(compactSlots(report.slots).map((slot) => slot.state)).toEqual(['success', 'missing']);
    expect(stateLabel('overlap')).toBe('Overlap');
  });

  it('explains malformed files', () => {
    expect(() => parseReport({ healthy: true })).toThrow('slots list');
    expect(() => parseReport({ slots: [{ job: 'x', scheduled_at: 'nope', state: 'success' }] })).toThrow('invalid expected time');
  });
});
