import fs from "node:fs/promises";
import { roles, elementRoles } from "aria-query";
import { AXObjects, elementAXObjects } from "axobject-query";

const abstractRoles = roles
  .keys()
  .sort()
  .filter((role) => roles.get(role).abstract);

const interactiveRoles = roles
  .keys()
  .sort()
  .filter(
    (name) =>
      !roles.get(name).abstract &&
      // The `progressbar` is descended from `widget`, but in practice, its
      // value is always `readonly`, so we treat it as a non-interactive role.
      name !== "progressbar" &&
      // This role is meant to have no semantic value.
      // @see https://www.w3.org/TR/wai-aria-1.2/#generic
      name !== "generic" &&
      roles.get(name).superClass.some((klasses) => klasses.includes("widget"))
  );
// 'toolbar' does not descend from widget, but it does support
// aria-activedescendant, thus in practice we treat it as a widget.
interactiveRoles.push("toolbar");

const nonInteractiveRoles = roles
  .keys()
  .sort()
  .filter(
    (name) =>
      !roles.get(name).abstract &&
      // 'toolbar' does not descend from widget, but it does support
      // aria-activedescendant, thus in practice we treat it as a widget.
      name !== "toolbar" &&
      // This role is meant to have no semantic value.
      // @see https://www.w3.org/TR/wai-aria-1.2/#generic
      name !== "generic" &&
      !roles.get(name).superClass.some((klasses) => klasses.includes("widget"))
  );
nonInteractiveRoles.push(
  // The `progressbar` is descended from `widget`, but in practice, its
  // value is always `readonly`, so we treat it as a non-interactive role.
  "progressbar"
);

const interactiveElementRoleSchemas = elementRoles
  .entries()
  .sort(([a], [b]) => a.name.localeCompare(b.name))
  .filter(([_, rolesArr]) =>
    rolesArr.some((role) => interactiveRoles.includes(role))
  )
  .map(([elementSchema]) => ({
    name: elementSchema.name,
    attributes: elementSchema.attributes || [],
  }));

const nonInteractiveElementRoleSchemas = elementRoles
  .entries()
  .sort(([a], [b]) => a.name.localeCompare(b.name))
  .filter(([_, rolesArr]) =>
    rolesArr.every((role) => nonInteractiveRoles.includes(role))
  )
  .map(([elementSchema]) => ({
    name: elementSchema.name,
    attributes: elementSchema.attributes || [],
  }));

const nonInteractiveAXObjects = new Set(
  AXObjects.entries()
    .filter(([_key, value]) => ["window", "structure"].includes(value.type))
    .map(([key, _value]) => key)
);

const nonInteractiveElementAXObjectSchemas = elementAXObjects
  .entries()
  .sort(([a], [b]) => a.name.localeCompare(b.name))
  .filter(([_, AXObjectsArr]) =>
    AXObjectsArr.every((role) => nonInteractiveAXObjects.has(role))
  )
  .map(([schema]) => ({
    name: schema.name,
    attributes: schema.attributes || [],
  }));

async function writeRoles() {
  await fs.writeFile(
    "abstractRoles.json",
    JSON.stringify(abstractRoles, null, 2)
  );

  await fs.writeFile(
    "interactiveRoles.json",
    JSON.stringify(interactiveRoles, null, 2)
  );

  await fs.writeFile(
    "noninteractiveRoles.json",
    JSON.stringify(nonInteractiveRoles, null, 2)
  );
}

async function writeElementSchema() {
  await fs.writeFile(
    "interactiveElementRoleSchemas.json",
    JSON.stringify(interactiveElementRoleSchemas, null, 2)
  );

  await fs.writeFile(
    "noninteractiveElementRoleSchemas.json",
    JSON.stringify(nonInteractiveElementRoleSchemas, null, 2)
  );
}

async function writeAxObjectElementSchema() {
  await fs.writeFile(
    "noninteractiveAxObjectSchema.json",
    JSON.stringify(nonInteractiveElementAXObjectSchemas, null, 2)
  );
}

async function main() {
  await Promise.all([
    writeRoles(),
    writeElementSchema(),
    writeAxObjectElementSchema(),
  ]);
}

await main();
