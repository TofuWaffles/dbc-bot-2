import React, { useState } from 'react';
import Link from 'next/link';
interface SidebarProps {
  guildId: string;
  items: { id: string, name: string }[];
}

const Sidebar: React.FC<SidebarProps> = ({ guildId, items }) => {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <div className="flex">
      <div
        className={`bg-gray-800 text-white fixed h-screen transition-all duration-300 z-10 ${isOpen ? 'w-64' : 'w-16'} overflow-hidden`}
      >
        <div className="flex flex-col items-center space-y-6 pt-4">
          <button
            className="text-white focus:outline-none"
            onClick={() => setIsOpen(!isOpen)}
          >
            {isOpen ? (
              <svg
                className="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            ) : (
              <svg
                className="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 6h16M4 12h16m-7 6h7"
                />
              </svg>
            )}
          </button>

          {isOpen && (
            <div className="flex flex-col items-start w-full px-4">
              {items.map((item, index) => (
                <div key={index} className="w-full py-2 hover:bg-gray-700 rounded">
                  <Link
                    href={`../bracket/${guildId}/${item.id}`}
                    className="block w-full text-white"
                  >
                    {item.name}
                  </Link>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
      <div className={`flex-1 p-4 transition-all duration-300 ${isOpen ? 'ml-64' : 'ml-16'}`}>
      </div>
    </div>
  );
};

export default Sidebar;