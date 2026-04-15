import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";

const appWindow = getCurrentWindow();

export function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    const updateMaximized = async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    };

    updateMaximized();
    
    // Listen for resize events to update the maximize state
    const unlisten = appWindow.onResized(async () => {
      updateMaximized();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleMinimize = () => appWindow.minimize();
  const handleMaximize = async () => {
    await appWindow.toggleMaximize();
  };
  const handleClose = () => appWindow.close();

  return (
    <div
      data-tauri-drag-region
      className="h-8 bg-ctp-crust flex items-center justify-between select-none border-b border-ctp-overlay0"
    >
      <div className="flex items-center px-4 gap-2 pointer-events-none">
        <span className="text-xs font-bold text-ctp-subtext1">decentcom</span>
      </div>

      <div className="flex items-center h-full">
        <button
          onClick={handleMinimize}
          className="w-12 h-full flex items-center justify-center hover:bg-ctp-surface0 transition-colors text-ctp-text"
          title="Minimize"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="2" y="5.5" width="8" height="1" fill="currentColor" />
          </svg>
        </button>
        <button
          onClick={handleMaximize}
          className="w-12 h-full flex items-center justify-center hover:bg-ctp-surface0 transition-colors text-ctp-text"
          title={isMaximized ? "Restore" : "Maximize"}
        >
          {isMaximized ? (
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path
                d="M3 3h6v6H3V3zm1 1v4h4V4H4z"
                fill="currentColor"
                fillRule="evenodd"
              />
              <path
                d="M5 2h5v5H9V3H5V2z"
                fill="currentColor"
              />
            </svg>
          ) : (
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect
                x="3"
                y="3"
                width="6"
                height="6"
                rx="0.5"
                fill="none"
                stroke="currentColor"
              />
            </svg>
          )}
        </button>
        <button
          onClick={handleClose}
          className="w-12 h-full flex items-center justify-center hover:bg-ctp-red hover:text-ctp-crust transition-colors text-ctp-text"
          title="Close"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path
              d="M3 3l6 6m0-6L3 9"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
