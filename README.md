# Bunguette

Bunguette is a couch coop game made during the [TGC game jam](https://itch.io/jam/tgcxcoreblazer).

Download the executable file from [itch.io](https://chang-kah-boon.itch.io/bunguette)!

Alternatively, you can also play the [web version](https://nixon-voxell.github.io/bunguette/)!

[![bunguette](./.github/assets/bunguette.png)](https://youtu.be/cBMcfEZzbuU)

*Click image above for gameplay video on YouTube !*

## Gameplay

Feed the hungry! You and your partner have opened a generous shop that provides free food to starving animals.
But be warned—when they're not well-fed, these hungry animals become aggressive! Your job is to keep them full and satisfied.

### Core Mechanics

- Each level, you'll be given some raw ingredients and cooking appliances.
- You will need to build defense towers by cooking those raw ingredients using the appliances.
- Each tower has its own recipe and requires different amounts of ingredients.
- With the ingredients, you will be able to craft tower (instantly) and place them on the ground.
- Each well-fed enemy will in return provide you extra raw ingredients for you to build more towers.
- Towers have unlimited ammo but can be attacked by hungry animals.
- Enemies arrive in waves at fixed intervals.
- Since your character is made of food (a baguette and a polo bun), you can toss tiny versions of yourself (mini baguettes or polo buns) at enemies to keep them fed!

#### Level 1

- Enemy: Rats
- Raw ingredients: Corn
- Appliances: Rotisserie, Wok 
- Tower: Cannon Tower (Popcorn), Gun Tower (Roasted Corn)

#### How to win?

Survive all 3 waves in the level by preventing the hungry animals from reaching the blue portal!

## Technology Stack

| Component           | Tool/Library                                                                                             |
|---------------------|----------------------------------------------------------------------------------------------------------|
| Game Engine         | [Bevy](https://bevyengine.org/)                                                                          |
| Level Editor / Art  | [Bevy Skein](https://bevyskein.dev/) & [Blender](https://www.blender.org/)                               |
| Physics             | [Avian](https://github.com/Jondolf/avian)                                                                |
| Input Manager       | [Leafwing Input Manager](https://github.com/Leafwing-Studios/leafwing-input-manager)                     |
| Asset Loader        | [Bevy Asset Loader](https://github.com/NiklasEi/bevy_asset_loader)                                       |
| Audio               | [Bevy Seedling](https://github.com/corvusprudens/bevy_seedling)                                          |
| Outline             | [Bevy Mod Outline](https://github.com/komadori/bevy_mod_outline)                                         |
| Framepace           | [Bevy Framepace](https://github.com/aevyrie/bevy_framepace)                                              |
| Inspector           | [Bevy Inspector Egui](https://github.com/jakobhellermann/bevy-inspector-egui)                            |
