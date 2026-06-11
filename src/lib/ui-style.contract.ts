import { CHECKBOX_CLASS_NAME } from "@/lib/ui-style";

export const uiStyleContract = {
  checkboxClass: CHECKBOX_CLASS_NAME,
  usesYadigCheckbox: CHECKBOX_CLASS_NAME.includes("yadig-checkbox"),
  keepsCheckboxSizeStable: CHECKBOX_CLASS_NAME.includes("h-4") && CHECKBOX_CLASS_NAME.includes("w-4"),
  preventsCheckboxShrink: CHECKBOX_CLASS_NAME.includes("shrink-0"),
};
