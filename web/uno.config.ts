import { defineConfig, presetIcons, presetUno } from "unocss";
import { presetShadcn } from "unocss-preset-shadcn";

export default defineConfig({
    presets: [
        presetUno(),
        presetIcons(),
        presetShadcn({
            color: "neutral",
        }),
    ],
});
