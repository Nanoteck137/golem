import { Dialog } from "@headlessui/react";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

const Card = (props: { onClick?: () => void }) => {
  return (
    <div
      className="rounded-md bg-gradient-to-t from-pink-500 via-red-700 to-yellow-500 px-1 cursor-pointer"
      onClick={() => {
        if (props.onClick) props.onClick();
      }}
    >
      <div className="rounded-md flex flex-col h-full w-full items-center justify-center bg-black back p-8">
        <p className="text-white text-center">Machine 01</p>

        <div className="h-4"></div>

        <p className="text-white text-center">IP: 10.28.28.1</p>
        <p className="text-white text-center">CPU: 8 core(s)</p>
        <p className="text-white text-center">Memory: 8/16 GB</p>
      </div>
    </div>
  );
};

const ProgressBar = (props: { progress: number }) => {
  return (
    <div className="w-full h-2.5 rounded-sm bg-gray-700">
      <div
        className="rounded-sm h-2.5 bg-yellow-500"
        style={{ width: props.progress * 100 + "%" }}
      ></div>
    </div>
  );
};

const MachineInfoModal = (props: { open: boolean; close: () => void }) => {
  return (
    <Dialog open={props.open} onClose={props.close} className="relative z-50">
      <div className="fixed inset-0 bg-black/60" aria-hidden="true" />

      <div className="fixed inset-0 flex items-center justify-center p-4">
        <div className="mx-auto max-w-sm rounded-md bg-gradient-to-t from-pink-500 via-red-700 to-yellow-500">
          <Dialog.Panel className="rounded-md bg-black m-1 p-4">
            <Dialog.Title className="text-white">Machine 01</Dialog.Title>

            <Dialog.Description className="text-white">
              Machine info
            </Dialog.Description>

            <div className="h-4"></div>

            <div>
              <p className="text-white">IP: 10.28.28.1</p>
              <p className="text-white">CPU: Apple M1</p>
              <p className="text-white">
                Memory: {(17179869184 / 1024 / 1024 / 1024).toFixed(2)} GB
              </p>
            </div>

            <div className="grid grid-cols-2 gap-x-4 gap-y-2">
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 0: 10%</p>
                <ProgressBar progress={0.1} />
              </div>
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 1: 20%</p>
                <ProgressBar progress={0.2} />
              </div>
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 2: 40%</p>
                <ProgressBar progress={0.4} />
              </div>
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 3: 50%</p>
                <ProgressBar progress={0.5} />
              </div>
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 4: 60%</p>
                <ProgressBar progress={0.6} />
              </div>
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 5: 70%</p>
                <ProgressBar progress={0.9} />
              </div>
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 6: 80%</p>
                <ProgressBar progress={0.8} />
              </div>
              <div className="flex flex-col">
                <p className="text-white font-mono">CPU 7: 100%</p>
                <ProgressBar progress={1.0} />
              </div>
            </div>

            <div>
              <p className="text-white">Memory:</p>
              <p className="text-white">Usage: 8 GB / 16 GB</p>
              <ProgressBar progress={0.5} />
              <p className="text-white">Swap: 2 GB / 4 GB</p>
            </div>

            <div className="h-8"></div>

            <button
              onClick={() => props.close()}
              className="bg-pink-400 px-4 py-1 rounded-md text-black"
            >
              Close
            </button>
          </Dialog.Panel>
        </div>
      </div>
    </Dialog>
  );
};

async function fetchMachines() {
  const data = await fetch("http://localhost:8000/api/system");
  return await data.json();
}

export const App = () => {
  const [isOpen, setOpen] = useState(false);

  const { data } = useQuery({ queryKey: ["machines"], queryFn: fetchMachines });
  console.log(data);

  return (
    <div className="">
      <div className="grid grid-cols-4 gap-8 p-8">
        {Array.from(Array(16).keys()).map((value) => {
          return (
            <Card
              key={value}
              onClick={() => {
                setOpen(true);
              }}
            />
          );
        })}
      </div>

      <MachineInfoModal
        open={isOpen}
        close={() => {
          setOpen(false);
        }}
      />
    </div>
  );
};
