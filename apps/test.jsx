export default function App() {
  return (
    <div className="bg-gray-50 min-h-screen text-gray-900">
      <main className="mx-auto max-w-3xl px-6 py-16">
        <div className="rounded-2xl bg-white p-8 shadow-sm ring-1 ring-black/5">
          <h1 className="text-3xl font-semibold tracking-tight">Tailwind JSX Starter</h1>
          <p className="mt-3 text-gray-600">
            This component uses Tailwind utility classes. Make sure Tailwind is configured in your
            build (Vite/Next/etc.).
          </p>

          <div className="mt-8 flex flex-wrap gap-3">
            <button
              type="button"
              className="inline-flex items-center justify-center rounded-xl bg-black px-4 py-2 text-sm font-medium text-white hover:bg-black/90"
            >
              Primary button
            </button>
            <button
              type="button"
              className="inline-flex items-center justify-center rounded-xl bg-white px-4 py-2 text-sm font-medium text-gray-900 ring-1 ring-gray-200 hover:bg-gray-50"
            >
              Secondary button
            </button>
          </div>

          <div className="mt-10 grid gap-4 sm:grid-cols-2">
            <div className="rounded-2xl bg-gray-50 p-5 ring-1 ring-gray-200">
              <p className="text-sm font-medium">Card A</p>
              <p className="mt-1 text-sm text-gray-600">A simple Tailwind card.</p>
            </div>
            <div className="rounded-2xl bg-gray-50 p-5 ring-1 ring-gray-200">
              <p className="text-sm font-medium">Card B</p>
              <p className="mt-1 text-sm text-gray-600">Another one, same vibes.</p>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
