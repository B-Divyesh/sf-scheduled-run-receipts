import '@fontsource-variable/instrument-sans/wght.css';
import '@fontsource/ibm-plex-mono/latin-500.css';
import './style.css';
import { compactSlots, parseReport, stateLabel, type CheckReport, type Slot } from './receipt';

const demo: CheckReport = {
  generated_at: '2026-08-28T08:16:00Z',
  since: '2026-08-22T00:00:00Z',
  healthy: false,
  counts: { success: 5, missing: 1, late: 1 },
  slots: [
    ['database-backup', '2026-08-22T02:00:00Z', 'success', 'backup-0822'],
    ['database-backup', '2026-08-23T02:00:00Z', 'success', 'backup-0823'],
    ['database-backup', '2026-08-24T02:00:00Z', 'late', 'backup-0824'],
    ['database-backup', '2026-08-25T02:00:00Z', 'success', 'backup-0825'],
    ['database-backup', '2026-08-26T02:00:00Z', 'missing', undefined],
    ['database-backup', '2026-08-27T02:00:00Z', 'success', 'backup-0827'],
    ['database-backup', '2026-08-28T02:00:00Z', 'success', 'backup-0828'],
  ].map(([job, scheduled_at, state, run_id]) => ({ job, scheduled_at, state, run_id } as Slot)),
};

const timeline = document.querySelector<HTMLElement>('#timeline');
const detail = document.querySelector<HTMLElement>('#receipt-detail');
const summary = document.querySelector<HTMLElement>('#viewer-summary');
const fileInput = document.querySelector<HTMLInputElement>('#report-file');
const fileStatus = document.querySelector<HTMLElement>('#file-status');

function formatDay(value: string) {
  return new Intl.DateTimeFormat('en', { weekday: 'short', timeZone: 'UTC' }).format(new Date(value));
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat('en', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit', hour12: false, timeZone: 'UTC', timeZoneName: 'short' }).format(new Date(value));
}

function selectSlot(button: HTMLButtonElement, slot: Slot) {
  timeline?.querySelectorAll<HTMLButtonElement>('[role="radio"]').forEach((node) => {
    node.setAttribute('aria-checked', 'false');
    node.tabIndex = -1;
  });
  button.setAttribute('aria-checked', 'true');
  button.tabIndex = 0;
  if (!detail) return;
  const copy = slot.state === 'missing'
    ? 'No signed start arrived before the 15 minute grace window. The absence was recorded at 02:15 UTC.'
    : slot.state === 'late'
      ? `Start receipt ${slot.run_id ?? ''} arrived after the grace window.`
      : `${stateLabel(slot.state)} receipt ${slot.run_id ?? ''} is linked to this expected slot.`;
  detail.innerHTML = `<span class="state-mark state-${slot.state}">${stateLabel(slot.state)}</span><strong>${slot.job}</strong><span>${formatTime(slot.scheduled_at)}</span><p>${copy}</p>`;
}

function renderReport(report: CheckReport) {
  if (!timeline || !summary) return;
  const slots = compactSlots(report.slots);
  timeline.replaceChildren();
  if (!slots.length) {
    timeline.innerHTML = '<p class="viewer-empty">No expected slots in this report. Add a job, then run <code>srr check --json</code> again.</p>';
    detail!.innerHTML = '<strong>Nothing due yet</strong><p>The viewer is ready when the first expected slot appears.</p>';
    summary.textContent = '0 expected slots';
    return;
  }
  const exceptions = slots.filter((slot) => !['success', 'pending'].includes(slot.state)).length;
  summary.textContent = `${slots.length} slots · ${exceptions} ${exceptions === 1 ? 'exception' : 'exceptions'}`;
  slots.forEach((slot, index) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = `slot slot-${slot.state}`;
    button.setAttribute('role', 'radio');
    button.setAttribute('aria-checked', String(index === slots.length - 1));
    button.tabIndex = index === slots.length - 1 ? 0 : -1;
    button.innerHTML = `<span class="slot-node" aria-hidden="true"></span><span>${formatDay(slot.scheduled_at)}</span><small>${stateLabel(slot.state)}</small>`;
    button.addEventListener('click', () => selectSlot(button, slot));
    button.addEventListener('keydown', (event) => {
      if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
      event.preventDefault();
      const buttons = [...timeline.querySelectorAll<HTMLButtonElement>('[role="radio"]')];
      const next = event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? buttons.length - 1
          : (buttons.indexOf(button) + (event.key === 'ArrowRight' ? 1 : -1) + buttons.length) % buttons.length;
      const nextButton = buttons[next];
      if (!nextButton) return;
      nextButton.focus();
      nextButton.click();
    });
    timeline.append(button);
  });
  selectSlot(timeline.lastElementChild as HTMLButtonElement, slots.at(-1)!);
}

fileInput?.addEventListener('change', async () => {
  const file = fileInput.files?.[0];
  if (!file || !fileStatus) return;
  fileStatus.textContent = `Reading ${file.name}…`;
  try {
    const report = parseReport(JSON.parse(await file.text()));
    renderReport(report);
    fileStatus.textContent = `Loaded ${file.name} locally. Nothing was uploaded.`;
  } catch (error) {
    fileStatus.textContent = error instanceof Error ? `${error.message} Choose JSON produced by srr check --json.` : 'Could not read that report.';
  }
});

document.querySelectorAll<HTMLButtonElement>('[data-copy]').forEach((button) => {
  button.addEventListener('click', async () => {
    const text = button.dataset.copy ?? '';
    try {
      await navigator.clipboard.writeText(text);
      const original = button.textContent;
      button.textContent = 'Copied';
      setTimeout(() => { button.textContent = original; }, 1600);
    } catch {
      button.textContent = 'Select command below';
      document.querySelector<HTMLElement>('#install-command')?.focus();
    }
  });
});

const networkStatus = document.querySelector<HTMLElement>('#network-status');
function updateNetworkStatus() {
  if (!networkStatus) return;
  const offline = !navigator.onLine;
  networkStatus.hidden = !offline;
  networkStatus.textContent = offline ? 'Offline — docs and the local report viewer still work.' : '';
}
window.addEventListener('online', updateNetworkStatus);
window.addEventListener('offline', updateNetworkStatus);
updateNetworkStatus();
renderReport(demo);

if ('serviceWorker' in navigator && import.meta.env.PROD) navigator.serviceWorker.register('/sw.js').catch(() => undefined);
