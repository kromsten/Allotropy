<script lang="ts">
	const { name, symbol, category, impact, tvl, apy, health } = $props();
</script>

<div class="bg-surface-900 border border-surface-800 rounded-3xl p-6 hover:border-primary-500/30 transition-all group">
	<div class="flex justify-between items-start mb-5">
		<div class="text-5xl">{symbol}</div>
		<div class="px-3 py-1 text-xs rounded-full bg-surface-800 text-surface-300">
			{category}
		</div>
	</div>

	<div class="mb-5">
		<div class="font-semibold text-xl tracking-tight">{name}</div>
	</div>

	<div class="space-y-3 text-sm mb-6">
		<div class="flex justify-between">
			<span class="text-surface-400">Verified Impact</span>
			<span class="font-medium text-success-400">{impact}</span>
		</div>
		<div class="flex justify-between">
			<span class="text-surface-400">TVL</span>
			<span class="font-medium text-highlight-400">{tvl}</span>
		</div>
		<div class="flex justify-between">
			<span class="text-surface-400">Staking APY</span>
			<span class="font-medium">{apy}</span>
		</div>
	</div>

	<div class="mb-5">
		<div class="flex justify-between text-xs mb-1.5">
			<span class="text-surface-400">Regeneration Health</span>
			<span>{health}%</span>
		</div>
		<div class="h-2 bg-surface-800 rounded-full overflow-hidden">
			<div class="h-full bg-linear-to-r from-primary-500 to-secondary-500 transition-all" 
				 style="width: {health}%"></div>
		</div>
	</div>

	<button class="btn preset-filled-primary w-full">
		Stake & Regenerate
	</button>
</div>