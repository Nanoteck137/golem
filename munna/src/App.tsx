import { Menu } from "@headlessui/react";

const Card = (props: { onClick?: () => void }) => {
  return (
    <a
      href="#"
      className="rounded-md bg-gradient-to-t from-pink-500 via-red-700 to-yellow-500 px-1"
      onClick={() => {
        if (props.onClick) props.onClick();
      }}
    >
      <div className="flex flex-col h-full w-full items-center justify-center bg-black back p-8">
        <p className="text-white text-center">Machine 01</p>

        <div className="h-4"></div>

        <p className="text-white text-center">IP: 10.28.28.1</p>
        <p className="text-white text-center">CPU: 8 core(s)</p>
        <p className="text-white text-center">Memory: 8/16 GB</p>
      </div>
    </a>
  );
};

function MyDropdown() {
  return (
    <Menu>
      <Menu.Button className="text-white">More</Menu.Button>
      <Menu.Items>
        <Menu.Item>
          {({ active }) => (
            <a className={`${active && "bg-blue-500"}`} href="#">
              Account settings
            </a>
          )}
        </Menu.Item>
        <Menu.Item>
          {({ active }) => (
            <a className={`${active && "bg-blue-500"}`} href="#">
              Documentation
            </a>
          )}
        </Menu.Item>
        <Menu.Item disabled>
          <span className="opacity-75">Invite a friend (coming soon!)</span>
        </Menu.Item>
      </Menu.Items>
    </Menu>
  );
}

export function App() {
  return (
    <div className="min-h-screen bg-black">
      <div className="flex justify-around">
        {Array.from(Array(4).keys()).map((value) => {
          return <Card key={value} />;
        })}
      </div>
      <MyDropdown />
    </div>
  );
}
