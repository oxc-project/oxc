import { Canvas } from '@react-three/fiber';
// PR status
debugger;

export function ThreeDMain() {
  return (
    <Canvas
      frameloop="demand"
      resize={{ debounce: 2 }}
      shadows
      gl={{ antialias: true, preserveDrawingBuffer: true }}
      camera={{
        position: [1.5, 0.5, 3.2],
        fov: 10,
        near: 0.1,
        far: 50,
      }}
      key={1}
    >
    </Canvas>
  )
}

