import { useEffect } from "react";
import { Gesture } from "react-native-gesture-handler";
import { useSharedValue, withSpring, withTiming } from "react-native-reanimated";

// Shared values imported from Reanimated are ref-like, so writing them outside
// of render is allowed.
export function SomeComponent() {
  const sharedVal = useSharedValue(0);
  return <Button onPress={() => (sharedVal.value = Math.random())} title="Randomize" />;
}

export function usePressToGrow() {
  const width = useSharedValue(100);
  return () => {
    width.value = withSpring(width.value + 50);
  };
}

export function useFadeIn(visible: boolean) {
  const opacity = useSharedValue(0);
  useEffect(() => {
    opacity.value = withTiming(visible ? 1 : 0);
  }, [opacity, visible]);
  return opacity;
}

export function useDrag() {
  const offset = useSharedValue(0);
  return Gesture.Pan()
    .onChange((event) => {
      offset.value += event.changeX;
    })
    .onFinalize(() => {
      offset.value = withSpring(0);
    });
}
