import mermaid from "@ooker777/lume-mermaid-plugin/";
import lume from "lume/mod.ts";
import lumocs from "lumocs/mod.ts";

const site = lume({
    src: "src",
});

site.use(mermaid());
site.use(lumocs());

export default site;
