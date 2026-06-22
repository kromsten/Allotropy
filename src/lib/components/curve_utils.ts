import type { Chart, Plugin, PluginOptionsByType } from 'chart.js';

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





export function generateAreaUnderCurve(
  a: number,
  b: number,
  toSupply: number,
  steps = 60
) {
  if (toSupply <= 0) return [];
 
  const points: { x: number; y: number }[] = [{ x: 0, y: b }];
  for (let i = 1; i <= steps; i++) {
    const s = (toSupply / steps) * i;
    points.push({ x: s, y: a * Math.sqrt(Math.max(0, s)) + b });
  }
  return points;
}
 
export function createHoverCrosshairPlugin(a: number, b: number): Plugin<'line'> {
  let hoverSupply: number | null = null;
 
  return {
    id: 'hoverCrosshair',
    afterEvent(chart, args) {
      const { event, inChartArea } = args;
 
      if (event.type === 'mousemove' && inChartArea) {
        const supply = chart.scales.x.getValueForPixel(event.x!);
        if (typeof supply === 'number' && Number.isFinite(supply)) {
          const clamped = Math.max(0, Math.min(supply, chart.scales.x.max as number));
          if (hoverSupply !== clamped) {
            hoverSupply = clamped;
            args.changed = true;
          }
        }
      } else if (
        event.type === 'mouseout' ||
        (event.type === 'mousemove' && !inChartArea && hoverSupply !== null)
      ) {
        hoverSupply = null;
        args.changed = true;
      }
    },
    beforeDatasetsDraw(chart) {
      if (hoverSupply === null) return;
 
      const { ctx, chartArea } = chart;
      const xScale = chart.scales.x;
      const yScale = chart.scales.y;
      const supply = hoverSupply;
      const price = a * Math.sqrt(Math.max(0, supply)) + b;
 
      const x = xScale.getPixelForValue(supply);
      const y = yScale.getPixelForValue(price);
      const yBottom = yScale.getPixelForValue(0);
      const xLeft = xScale.getPixelForValue(0);
 
      ctx.save();
 
      ctx.beginPath();
      ctx.moveTo(xLeft, yBottom);
 
      const steps = 50;
      for (let i = 0; i <= steps; i++) {
        const s = (supply / steps) * i;
        const p = a * Math.sqrt(Math.max(0, s)) + b;
        ctx.lineTo(xScale.getPixelForValue(s), yScale.getPixelForValue(p));
      }
 
      ctx.lineTo(x, yBottom);
      ctx.closePath();
      ctx.fillStyle = 'rgba(103, 232, 249, 0.18)';
      ctx.fill();
 
      ctx.beginPath();
      ctx.moveTo(x, y);
      ctx.lineTo(x, yBottom);
      ctx.strokeStyle = 'rgba(103, 232, 249, 0.75)';
      ctx.lineWidth = 1.5;
      ctx.setLineDash([5, 4]);
      ctx.stroke();
      ctx.setLineDash([]);
 
      ctx.beginPath();
      ctx.arc(x, y, 5, 0, Math.PI * 2);
      ctx.fillStyle = '#67e8f9';
      ctx.fill();
      ctx.strokeStyle = '#0a0a0f';
      ctx.lineWidth = 2;
      ctx.stroke();
 
      ctx.restore();
    }
  };
}
 