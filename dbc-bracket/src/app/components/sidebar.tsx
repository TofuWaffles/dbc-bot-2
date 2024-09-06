import React, { useState } from 'react';
import { useRouter } from 'next/navigation';

interface SidebarProps {
  items: string[];
}

const Sidebar: React.FC<SidebarProps> = ({ items }) => {
  const [isOpen, setIsOpen] = useState(false);
  const router = useRouter();

  const handleNavigation = (item: string) => {
    router.push(`/api/tournaments/${item}`);
  };

  return (
    <div className="flex">
      <div
        className={`bg-gray-800 text-white fixed h-screen transition-all duration-300 z-10 ${isOpen ? 'w-64' : 'w-0 overflow-hidden'}`}
      >
        <div className="flex flex-col items-center">
        {items.map((item, index) => {
          if (typeof item !== 'string') {
            console.warn('Non-string item detected:', item); // Log if an item is not a string
            return null; // Skip rendering invalid items
          }

          return (
            <div key={index}>
              <a href="#" onClick={() => handleNavigation(item)}>
                {item}
              </a>
            </div>
          );
        })}
        </div>
      </div>
      <div className={`flex-1 p-4 ${isOpen ? 'ml-64' : 'ml-0'}`}>
        <div className="ml-auto">
          <button
            className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
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
        </div>
      </div>
    </div>
  );
};

export default Sidebar;