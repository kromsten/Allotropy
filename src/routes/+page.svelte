<script lang="ts">
	import FancyButton from "$lib/components/FancyButton.svelte";
	import TokenCard from "$lib/components/TokenCard.svelte";

</script>

<!-- Navbar -->
<nav class="sticky top-0 z-50 border-b border-surface-800 bg-surface-950/95 backdrop-blur">
	<div class="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<div class="w-8 h-8 bg-primary-500 rounded-lg flex items-center justify-center">
				<span class="text-white font-bold text-xl">R</span>
			</div>
			<span class="font-semibold text-xl tracking-tight">RegenForge</span>
		</div>

		<div class="flex items-center gap-8 text-sm">
			<a href="/discover" class="hover:text-primary-400 transition-colors">Discover</a>
			<a href="/impact" class="hover:text-primary-400 transition-colors">Impact</a>
			<a href="/portfolio" class="hover:text-primary-400 transition-colors">Portfolio</a>
		</div>

		<FancyButton text="Connect Wallet" variant="secondary" />
	</div>
</nav>

<!-- Hero -->
<section class="max-w-4xl mx-auto px-6 pt-20 pb-16 text-center">
	<h1 class="text-6xl font-bold tracking-tighter mb-4">
		Regenerative Tokens<br>on Cosmos Hub
	</h1>
	<p class="text-xl text-surface-300 max-w-md mx-auto mb-8">
		Launch persistent tokens that create real-world impact.
	</p>

	<FancyButton 
		text="Forge New Token" 
		variant="premium" 
		size="lg"
		onclick={() => window.location.href = '/forge'}
	/>
</section>

<!-- Trust Metrics -->
<section class="max-w-6xl mx-auto px-6 pb-12">
	<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
		<div class="bg-surface-900 border border-surface-800 rounded-2xl p-6">
			<div class="text-surface-400 text-sm mb-1">Projects Live</div>
			<div class="text-4xl font-semibold">142</div>
		</div>
		<div class="bg-surface-900 border border-surface-800 rounded-2xl p-6">
			<div class="text-surface-400 text-sm mb-1">Total Impact</div>
			<div class="text-4xl font-semibold text-success-500">847k tCO₂</div>
		</div>
		<div class="bg-surface-900 border border-surface-800 rounded-2xl p-6">
			<div class="text-surface-400 text-sm mb-1">Total Value Locked</div>
			<div class="text-4xl font-semibold text-highlight-500">$18.4M</div>
		</div>
		<div class="bg-surface-900 border border-surface-800 rounded-2xl p-6">
			<div class="text-surface-400 text-sm mb-1">Active Stakers</div>
			<div class="text-4xl font-semibold">9,842</div>
		</div>
	</div>
</section>

<!-- Active Tokens -->
<section class="max-w-7xl mx-auto px-6 pb-20">
	<div class="flex items-center justify-between mb-6">
		<h2 class="text-3xl font-semibold tracking-tight">Active Regenerative Tokens</h2>
		<a href="/discover" class="text-primary-400 hover:underline text-sm">View all →</a>
	</div>

	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
		<TokenCard 
			name="Amazon Canopy"
			symbol="🌿"
			category="Reforestation"
			impact="+124,500 tCO₂"
			tvl="2.8M ATOM"
			apy="18.4%"
			health={87}
		/>
		<TokenCard 
			name="Sahara Solar"
			symbol="☀️"
			category="Renewable Energy"
			impact="+89,200 MWh"
			tvl="1.9M ATOM"
			apy="14.2%"
			health={92}
		/>
		<TokenCard 
			name="Coral Reef Fund"
			symbol="🌊"
			category="Ocean Health"
			impact="+31,400 ha"
			tvl="1.4M ATOM"
			apy="21.7%"
			health={74}
		/>
		<TokenCard 
			name="Biodiversity DAO"
			symbol="🦋"
			category="Biodiversity"
			impact="+47 species"
			tvl="980K ATOM"
			apy="16.8%"
			health={81}
		/>
	</div>
</section>