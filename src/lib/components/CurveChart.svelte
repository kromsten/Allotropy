<!-- src/lib/components/CurveChart.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    Chart,
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    Filler,
    Tooltip,
    type ChartConfiguration
  } from 'chart.js';

  import { resetZoom, zoomRect, zoomScale  } from 'chartjs-plugin-zoom';

  import type { ChartComponentProps } from '$types';

  import {
    generateCurveData,
    generatePurchaseArea,
    getChartPluginOptions,
    getFocusedRange
  } from './curve_utils';

  Chart.register(LineController, LineElement, PointElement, LinearScale, Filler, Tooltip);

  let {
    a = 0.03,
    b = 0,
    previewSupply,
    currentSupply = $bindable(245_678),
    maxSupply = 1_000_000,
    height = 420,
    isZoomed = false,
  }: ChartComponentProps = $props();

  let canvas: HTMLCanvasElement;
  let chart: Chart | null = null;

  function applyZoomState() {
    if (!chart) return;

    const diff = (previewSupply ?? 0) - currentSupply;

    if (isZoomed && previewSupply && diff > 0) {

      zoomRect(chart, 
      { x: currentSupply - diff * 0.1, y: a * Math.sqrt(currentSupply) + b },
      { x: previewSupply + diff * 0.1, y: a * Math.sqrt(previewSupply) + b
      });


      const { min, max } = getFocusedRange(previewSupply, currentSupply, maxSupply);

      zoomScale(chart, 'x', { min, max });
      
    } else {
      resetZoom(chart);
    }
  }

  async function createChart() {
    if (!canvas || chart) return;

    try {
      const zoomModule = await import('chartjs-plugin-zoom');
      
      Chart.register(zoomModule.default);
    } catch (err) {
      console.warn('[CurveChart] chartjs-plugin-zoom failed to load', err);
    }

    const currentPrice = a * Math.sqrt(currentSupply) + b;
    const previewPrice = previewSupply ? a * Math.sqrt(previewSupply) + b : currentPrice;

    const config: ChartConfiguration = {
      type: 'line',
      data: {
        datasets: [
          {
            label: 'Bonding Curve',
            data: generateCurveData(a, b, maxSupply),
            borderColor: '#67e8f9',
            borderWidth: 3.5,
            tension: 0.38,
            fill: false,
            pointRadius: 0
          },
          {
            label: 'Current Supply',
            data: [{ x: currentSupply, y: currentPrice }],
            borderColor: '#10b981',
            backgroundColor: '#10b981',
            pointRadius: 8,
            pointHoverRadius: 11,
            showLine: false
          },
          {
            label: 'After Purchase',
            data: previewSupply && previewSupply > currentSupply
              ? [{ x: previewSupply, y: previewPrice }]
              : [],
            borderColor: '#c026ff',
            backgroundColor: '#c026ff',
            pointRadius: 7,
            showLine: false,
            hidden: !previewSupply || previewSupply <= currentSupply
          },
          {
            label: 'Your Purchase',
            data: isZoomed && previewSupply
              ? generatePurchaseArea(a, b, currentSupply, previewSupply)
              : [],
            borderColor: '#67e8f9',
            backgroundColor: 'rgba(103, 232, 249, 0.32)',
            borderWidth: 2,
            fill: true,
            tension: 0.38,
            pointRadius: 0,
            hidden: !isZoomed || !previewSupply || previewSupply <= currentSupply
          }
        ]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: { duration: 400, easing: 'easeOutQuart' },
        scales: {
          x: {
            type: 'linear',
            title: { display: true, text: 'Total Supply (BOND)', color: '#ffffff80', font: { size: 11 } },
            grid: { color: '#ffffff12' },
            ticks: { color: '#ffffff50', font: { size: 10 } }
          },
          y: {
            type: 'linear',
            title: { display: true, text: 'Price per BOND (ATOM)', color: '#ffffff80', font: { size: 11 } },
            grid: { color: '#ffffff12' },
            ticks: { color: '#ffffff50', font: { size: 10 } }
          },
        },
        plugins: getChartPluginOptions(a, b, maxSupply)
      }
    };

    chart = new Chart(canvas, config);
    setTimeout(() => applyZoomState(), 80);
  }

  function updateChart() {
    if (!chart) return;

    const currentPrice = a * Math.sqrt(currentSupply) + b;
    const previewPrice = previewSupply ? a * Math.sqrt(previewSupply) + b : currentPrice;
    const hasPreview = previewSupply && previewSupply > currentSupply;

    chart.data.datasets[0].data = generateCurveData(a, b, maxSupply);
    chart.data.datasets[1].data = [{ x: currentSupply, y: currentPrice }];

    chart.data.datasets[2].data = hasPreview ? [{ x: previewSupply!, y: previewPrice }] : [];
    chart.data.datasets[2].hidden = !hasPreview;

    chart.data.datasets[3].data = (isZoomed && hasPreview)
      ? generatePurchaseArea(a, b, currentSupply, previewSupply!)
      : [];
    chart.data.datasets[3].hidden = !isZoomed || !hasPreview;

    chart.update('none');
  }

  $effect(() => {
    if (!chart) return;
    updateChart();
    applyZoomState();
  });

  $effect(() => {
    if (isZoomed || !isZoomed) applyZoomState();
  });

  onMount(() => createChart());

  onDestroy(() => {
    chart?.destroy();
    chart = null;
  });
</script>

<div class="relative w-full rounded-3xl overflow-hidden bg-[#0a0a0f]" style="height: {height}px">
  <canvas bind:this={canvas} class="w-full h-full"></canvas>
</div>