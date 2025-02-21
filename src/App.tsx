import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { openUrl } from '@tauri-apps/plugin-opener';

function App() {
  const [isRecording, setIsRecording] = useState(false)
  const [elapsedTime, setElapsedTime] = useState(0)
  const [osInfo, setOsInfo] = useState("")

  useEffect(() => {
    let interval: number | null = null
    if (isRecording) {
      interval = window.setInterval(() => {
        setElapsedTime((prevTime) => prevTime + 1)
      }, 1000)
    } else if (!isRecording && elapsedTime !== 0) {
      if (interval) clearInterval(interval)
    }
    return () => {
      if (interval) clearInterval(interval)
    }
  }, [isRecording, elapsedTime])

  useEffect(() => {
    invoke<string>("get_os_info").then(setOsInfo)
  }, [])

  const handleOpenBrowser = async () => {
    try {
      await openUrl("https://www.google.com")
    } catch (error) {
      console.error("Failed to open browser:", error)
    }
  }

  const handleCreateTask = async () => {
    try {
      if (!isRecording) {
        setIsRecording(true)
        setElapsedTime(0)
        await invoke("start_recording")
      } else {
        setIsRecording(false)
        await invoke("stop_recording")
      }
    } catch (error) {
      console.error("Failed to start/stop recording:", error)
    }
  }

  return (
    <div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center">
      <div className="bg-white p-8 rounded-lg shadow-md">
        <h1 className="text-2xl font-bold mb-6 text-center">Task Recorder</h1>
        <p className="text-center mb-4">OS: {osInfo}</p>
        <div className="space-y-4">
          <button
            className="w-full bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded"
            onClick={handleOpenBrowser}
          >
            Open Browser
          </button>
          <button
            className={`w-full font-bold py-2 px-4 rounded ${
              isRecording ? "bg-red-500 hover:bg-red-600 text-white" : "bg-green-500 hover:bg-green-600 text-white"
            }`}
            onClick={handleCreateTask}
          >
            {isRecording ? "Stop Recording" : "Create Task"}
          </button>
          {isRecording && <p className="text-center">Recording... Time elapsed: {elapsedTime} seconds</p>}
        </div>
      </div>
    </div>
  )
}

export default App;