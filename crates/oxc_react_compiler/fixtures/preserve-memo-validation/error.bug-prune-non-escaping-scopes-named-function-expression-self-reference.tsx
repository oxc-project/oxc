// A named function expression can capture its own binding in a nested function.
// Pruning must register that binding before following captured dependencies.
// The component must still bail out when manual memoization cannot be preserved.
import { memo, useCallback } from 'react';

type ObjectiveID = number;
type LevelData = {
  mapId: string;
  next?: ReadonlyArray<LevelData | readonly [ObjectiveID, LevelData]>;
};
type Props = {
  level: LevelData;
  maps: ReadonlyMap<string, { name: string }>;
  parentLevel?: LevelData;
  updateLevel: (level: unknown) => void;
};

export default memo(function Level({ level, parentLevel, ...commonProps }: Props) {
  const { maps, updateLevel } = commonProps;
  const node = maps.get(level.mapId)!;

  const updateObjective = useCallback(
    (objectiveId: ObjectiveID | null) => {
      if (parentLevel) {
        updateLevel({
          ...parentLevel,
          next: [...(parentLevel.next || [])].map((entry) => {
            const isArray = Array.isArray(entry);
            const { mapId } = isArray ? entry[1] : entry;
            // Only mutate if the level id matches.
            if (mapId === level.mapId) {
              return objectiveId != null ? [objectiveId, mapId] : mapId;
            }
            return isArray ? [entry[0], mapId] : mapId;
          }),
        });
      }
    },
    [level.mapId, parentLevel, updateLevel],
  );

  return (
    <>
      {getMapName(node.name)}
      {level.next?.map(() => <Level level={level} {...commonProps} />)}
    </>
  );
});
