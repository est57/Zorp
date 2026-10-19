import js from "@eslint/js";
import tseslint from "typescript-eslint";
import vue from "eslint-plugin-vue";
import prettier from "eslint-config-prettier";
import globals from "globals";

export default tseslint.config(
  { ignores: ["dist", "src-tauri", "src/bindings", "node_modules", "design-mockup.html"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...vue.configs["flat/recommended"],
  {
    files: ["**/*.vue"],
    languageOptions: { parserOptions: { parser: tseslint.parser } },
  },
  {
    languageOptions: { globals: { ...globals.browser } },
    rules: {
      "vue/no-v-html": "error",
      // Request drafts are shared reactive objects; only reassigning the prop itself is disallowed.
      "vue/no-mutating-props": ["error", { shallowOnly: true }],
      "vue/multi-word-component-names": "off",
      "max-lines": ["warn", { max: 200, skipBlankLines: true, skipComments: true }],
    },
  },
  prettier,
);
