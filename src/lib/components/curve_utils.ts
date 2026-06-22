import type { PluginOptionsByType } from 'chart.js';

export function generateCurveData(a: number, b: number, maxSupply: number) {
    const points: { x: number; y: number }[] = [];
    const steps = 110;
    for (let i = 0; i <= steps; i++) {
        const s = (maxSupply / steps) * i;
        const price = a * Math.sqrt(Math.max(0, s)) + b;
        points.push({ x: s, y: price });
    }
    return points;
}

export function generatePurchaseArea(a: number, b: number, from: number, to: number) {
  if (!to || to <= from) return [];
    const points: { x: number; y: number }[] = [];
    const steps = 40;
    const step = (to - from) / steps;
    for (let i = 0; i <= steps; i++) {
      const s = from + step * i;
      const price = a * Math.sqrt(Math.max(0, s)) + b;
      points.push({ x: s, y: price });
    }
    return points;
}



export const getChartPluginOptions = (a: number, b: number, maxSupply: number) : PluginOptionsByType<'line'>  => ({
    tooltip: {
        backgroundColor: '#0f0f14',
        borderColor: '#67e8f9',
        borderWidth: 1,
        displayColors: false,
        // @ts-ignore
        callbacks: {
            title: (ctx: any) => `Supply: ${ctx[0].raw.x.toLocaleString()} BOND`,
            label: (ctx: any) => `Price: ${ctx.raw.y.toFixed(4)} ATOM`
        }
    },
    zoom: {
        limits: {
            x: { min: 0, max: maxSupply },
            y: { min: 0, max: a * Math.sqrt(maxSupply) + b }
        },
        zoom: {
            wheel: { enabled: true },
            pinch: { enabled: true },
            mode: 'x'
        },
        pan: {
            enabled: true,
            mode: 'x'
        }
    }
})


export function getFocusedRange(
  previewSupply: number,
  currentSupply: number,
  maxSupply: number
) {
  if (!previewSupply || previewSupply <= currentSupply) {
    return { min: 0, max: maxSupply };
  }

  const purchaseRange = previewSupply - currentSupply;

  // Make the purchase area take ~30% of the visible chart width
  let visibleWidth = purchaseRange / 0.30;

  // Only apply minimum floor for very tiny purchases (prevents "thin vertical line")
  const minVisibleWidth = currentSupply * 0.065; // ~6.5%
  visibleWidth = Math.max(visibleWidth, minVisibleWidth);

  const padding = visibleWidth * 0.18;

  return {
    min: Math.max(0, currentSupply - padding),
    max: Math.min(maxSupply, previewSupply + padding * 1.4)
  };
}