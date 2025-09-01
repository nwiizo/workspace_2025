"use client";

import React, { useState, useEffect, useCallback } from "react";
import { 
  Play, Pause, RotateCcw, Timer, Plus, Check, X, Edit2, Trash2, 
  Clock, CheckCircle2, Circle, ChevronRight, AlertCircle, 
  ArrowUp, ArrowRight, ArrowDown, Hash, Calendar, Target, 
  TrendingUp, BarChart3, Activity, GitBranch, Coffee, 
  Zap, Flag, Filter, Search, MoreVertical, Settings,
  Moon, Sun, Archive, FolderOpen, Star,
  Download, Upload, Copy, ClipboardCheck, FileJson,
  Keyboard, HelpCircle, Command, GripVertical, Maximize2, Minimize2,
  Save, BookOpen, Share2, Link
} from "lucide-react";

type TimerMode = "work" | "shortBreak" | "longBreak";
type Priority = "high" | "medium" | "low";
type ViewMode = "board" | "list" | "calendar";

type Todo = {
  id: string;
  text: string;
  completed: boolean;
  priority: Priority;
  pomodorosCompleted: number;
  pomodorosEstimated: number;
  createdAt: Date;
  completedAt?: Date;
  tags: string[];
  project?: string;
  dueDate?: Date;
  order?: number;
};

type TaskTemplate = {
  id: string;
  name: string;
  text: string;
  priority: Priority;
  pomodorosEstimated: number;
  tags: string[];
  usageCount: number;
};

type Stats = {
  todayPomodoros: number;
  weekPomodoros: number;
  totalTasks: number;
  completedTasks: number;
  streak: number;
  dailyHistory?: Record<string, number>;
  weeklyHistory?: Record<string, number>;
  monthlyHistory?: Record<string, number>;
};

const TIMER_SETTINGS = {
  work: { 
    minutes: 25, 
    label: "Focus Session", 
    icon: Zap,
    color: "text-red-500",
    bg: "bg-red-50 dark:bg-red-950/20"
  },
  shortBreak: { 
    minutes: 5, 
    label: "Short Break", 
    icon: Coffee,
    color: "text-green-500",
    bg: "bg-green-50 dark:bg-green-950/20"
  },
  longBreak: { 
    minutes: 15, 
    label: "Long Break", 
    icon: Coffee,
    color: "text-blue-500",
    bg: "bg-blue-50 dark:bg-blue-950/20"
  },
};

const PRIORITY_CONFIG = {
  high: { 
    icon: ArrowUp, 
    color: "text-red-500 dark:text-red-400", 
    bg: "bg-red-100 dark:bg-red-900/20",
    label: "High Priority"
  },
  medium: { 
    icon: ArrowRight, 
    color: "text-yellow-500 dark:text-yellow-400", 
    bg: "bg-yellow-100 dark:bg-yellow-900/20",
    label: "Medium Priority"
  },
  low: { 
    icon: ArrowDown, 
    color: "text-gray-500 dark:text-gray-400", 
    bg: "bg-gray-100 dark:bg-gray-900/20",
    label: "Low Priority"
  },
};

export default function Home() {
  const [mode, setMode] = useState<TimerMode>("work");
  const [timeLeft, setTimeLeft] = useState(TIMER_SETTINGS.work.minutes * 60);
  const [isRunning, setIsRunning] = useState(false);
  const [completedPomodoros, setCompletedPomodoros] = useState(0);
  const [mounted, setMounted] = useState(false);
  const [darkMode, setDarkMode] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>("list");
  
  // TODO states
  const [todos, setTodos] = useState<Todo[]>([]);
  const [newTodo, setNewTodo] = useState("");
  const [selectedTodoId, setSelectedTodoId] = useState<string | null>(null);
  const [editingTodoId, setEditingTodoId] = useState<string | null>(null);
  const [editingText, setEditingText] = useState("");
  const [estimatedPomodoros, setEstimatedPomodoros] = useState(1);
  const [selectedPriority, setSelectedPriority] = useState<Priority>("medium");
  const [filterPriority, setFilterPriority] = useState<Priority | "all">("all");
  const [showCompleted, setShowCompleted] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedProject, setSelectedProject] = useState<string>("inbox");
  const [newTodoTags, setNewTodoTags] = useState("");
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [showTagView, setShowTagView] = useState(false);
  const [showExportImport, setShowExportImport] = useState(false);
  const [exportData, setExportData] = useState("");
  const [importData, setImportData] = useState("");
  const [copySuccess, setCopySuccess] = useState(false);
  const [showKeyboardHelp, setShowKeyboardHelp] = useState(false);
  const [draggedItem, setDraggedItem] = useState<Todo | null>(null);
  const [dragOverItem, setDragOverItem] = useState<string | null>(null);
  const [focusMode, setFocusMode] = useState(false);
  const [showStats, setShowStats] = useState(false);
  const [statsView, setStatsView] = useState<'daily' | 'weekly' | 'monthly'>('daily');
  const [templates, setTemplates] = useState<TaskTemplate[]>([]);
  const [showTemplates, setShowTemplates] = useState(false);
  const [shareUrl, setShareUrl] = useState<string>("");
  const [showShareModal, setShowShareModal] = useState(false);
  
  // Stats
  const [stats, setStats] = useState<Stats>({
    todayPomodoros: 0,
    weekPomodoros: 0,
    totalTasks: 0,
    completedTasks: 0,
    streak: 0
  });
  
  const currentSettings = TIMER_SETTINGS[mode];
  const CurrentIcon = currentSettings.icon;
  const selectedTodo = todos.find(t => t.id === selectedTodoId);

  useEffect(() => {
    setMounted(true);
    // Load from localStorage
    const savedTodos = localStorage.getItem("todos");
    const savedStats = localStorage.getItem("stats");
    const savedDarkMode = localStorage.getItem("darkMode");
    const savedTemplates = localStorage.getItem("templates");
    
    if (savedTodos) setTodos(JSON.parse(savedTodos));
    if (savedStats) setStats(JSON.parse(savedStats));
    if (savedDarkMode) setDarkMode(savedDarkMode === "true");
    if (savedTemplates) setTemplates(JSON.parse(savedTemplates));
  }, []);

  useEffect(() => {
    if (mounted) {
      localStorage.setItem("todos", JSON.stringify(todos));
    }
  }, [todos, mounted]);

  useEffect(() => {
    if (mounted) {
      localStorage.setItem("stats", JSON.stringify(stats));
    }
  }, [stats, mounted]);

  useEffect(() => {
    if (mounted) {
      localStorage.setItem("darkMode", darkMode.toString());
      if (darkMode) {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }
    }
  }, [darkMode, mounted]);

  useEffect(() => {
    let interval: NodeJS.Timeout | null = null;

    if (isRunning && timeLeft > 0) {
      interval = setInterval(() => {
        setTimeLeft((time) => time - 1);
      }, 1000);
    } else if (timeLeft === 0) {
      handleTimerComplete();
    }

    return () => {
      if (interval) clearInterval(interval);
    };
  }, [isRunning, timeLeft, mode]);

  const handleTimerComplete = () => {
    setIsRunning(false);
    
    if (mode === "work") {
      setCompletedPomodoros(prev => prev + 1);
      const today = new Date().toISOString().split('T')[0];
      const week = `${new Date().getFullYear()}-W${Math.ceil((new Date().getDate() + new Date(new Date().getFullYear(), new Date().getMonth(), 1).getDay()) / 7)}`;
      const month = new Date().toISOString().slice(0, 7);
      
      setStats(prev => {
        const newStats = {
          ...prev,
          todayPomodoros: prev.todayPomodoros + 1,
          weekPomodoros: prev.weekPomodoros + 1,
          dailyHistory: {
            ...prev.dailyHistory,
            [today]: (prev.dailyHistory?.[today] || 0) + 1
          },
          weeklyHistory: {
            ...prev.weeklyHistory,
            [week]: (prev.weeklyHistory?.[week] || 0) + 1
          },
          monthlyHistory: {
            ...prev.monthlyHistory,
            [month]: (prev.monthlyHistory?.[month] || 0) + 1
          }
        };
        localStorage.setItem('stats', JSON.stringify(newStats));
        return newStats;
      });
      
      if (selectedTodoId) {
        setTodos(todos.map(todo =>
          todo.id === selectedTodoId
            ? { ...todo, pomodorosCompleted: todo.pomodorosCompleted + 1 }
            : todo
        ));
      }
      
      const nextMode = completedPomodoros % 4 === 3 ? "longBreak" : "shortBreak";
      switchMode(nextMode);
    } else {
      switchMode("work");
    }

    playSound();
    showNotification();
  };

  const playSound = () => {
    const audio = new Audio("data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBTGH0fPTgjMGHm7A7+OZURE");
    audio.play().catch(() => {});
  };

  const showNotification = () => {
    if ("Notification" in window && Notification.permission === "granted") {
      new Notification("Pomodoro Complete!", {
        body: mode === "work" ? "Great work! Time for a break." : "Break's over! Ready to focus?",
        icon: "/favicon.ico",
      });
    }
  };

  const switchMode = (newMode: TimerMode) => {
    setMode(newMode);
    setTimeLeft(TIMER_SETTINGS[newMode].minutes * 60);
    setIsRunning(false);
  };

  const toggleTimer = () => {
    // 作業セッションの場合のみタスク選択を要求
    if (mode === "work" && !selectedTodoId && todos.filter(t => !t.completed).length > 0) {
      alert("タスクを選択してからタイマーを開始してください");
      return;
    }
    setIsRunning(!isRunning);
  };

  const resetTimer = () => {
    setIsRunning(false);
    setTimeLeft(currentSettings.minutes * 60);
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  const requestNotificationPermission = useCallback(async () => {
    if ("Notification" in window && Notification.permission === "default") {
      await Notification.requestPermission();
    }
  }, []);

  useEffect(() => {
    requestNotificationPermission();
  }, [requestNotificationPermission]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      // テキスト入力中は無効
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      // Cmd/Ctrl + K: キーボードショートカットヘルプ
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setShowKeyboardHelp(!showKeyboardHelp);
        return;
      }

      // Space: タイマー開始/停止
      if (e.key === ' ') {
        e.preventDefault();
        toggleTimer();
      }
      
      // R: リセット
      if (e.key === 'r' || e.key === 'R') {
        resetTimer();
      }
      
      // 1, 2, 3: モード切り替え
      if (e.key === '1') {
        switchMode('work');
      } else if (e.key === '2') {
        switchMode('shortBreak');
      } else if (e.key === '3') {
        switchMode('longBreak');
      }
      
      // N: 新しいタスク (フォーカス)
      if (e.key === 'n' || e.key === 'N') {
        const taskInput = document.querySelector('input[placeholder*="タスク"]') as HTMLInputElement;
        if (taskInput) {
          taskInput.focus();
        }
      }
      
      // D: ダークモード切り替え
      if (e.key === 'd' || e.key === 'D') {
        setDarkMode(!darkMode);
      }
      
      // E: エクスポート/インポート
      if (e.key === 'e' || e.key === 'E') {
        handleExport();
      }
      
      // /: 検索
      if (e.key === '/') {
        e.preventDefault();
        const searchInput = document.querySelector('input[placeholder*="Search"]') as HTMLInputElement;
        if (searchInput) {
          searchInput.focus();
        }
      }
      
      // F: 集中モード切り替え
      if (e.key === 'f' || e.key === 'F') {
        toggleFocusMode();
      }
      
      // S: 統計表示
      if (e.key === 's' || e.key === 'S') {
        setShowStats(!showStats);
      }
      
      // Escape: モーダルを閉じる/集中モード終了
      if (e.key === 'Escape') {
        if (focusMode) {
          setFocusMode(false);
        } else {
          setShowExportImport(false);
          setShowKeyboardHelp(false);
          setShowStats(false);
        }
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [darkMode, showKeyboardHelp, focusMode, showStats]);

  // TODO functions
  const addTodo = () => {
    if (newTodo.trim()) {
      // タグをカンマまたはスペースで分割して配列に変換
      const tagArray = newTodoTags
        .split(/[,\s]+/)
        .filter(tag => tag.trim())
        .map(tag => tag.trim().toLowerCase());
      
      const maxOrder = todos.reduce((max, t) => Math.max(max, t.order || 0), 0);
      const todo: Todo = {
        id: Date.now().toString(),
        text: newTodo,
        completed: false,
        priority: selectedPriority,
        pomodorosCompleted: 0,
        pomodorosEstimated: estimatedPomodoros,
        createdAt: new Date(),
        tags: tagArray,
        project: selectedProject,
        order: maxOrder + 1
      };
      setTodos([todo, ...todos]);
      setNewTodo("");
      setNewTodoTags("");
      setEstimatedPomodoros(1);
      setSelectedPriority("medium");
    }
  };

  const toggleTodo = (id: string) => {
    setTodos(todos.map(todo =>
      todo.id === id 
        ? { 
            ...todo, 
            completed: !todo.completed,
            completedAt: !todo.completed ? new Date() : undefined
          } 
        : todo
    ));
    
    if (!todos.find(t => t.id === id)?.completed) {
      setStats(prev => ({
        ...prev,
        completedTasks: prev.completedTasks + 1
      }));
    }
  };

  const deleteTodo = (id: string) => {
    setTodos(todos.filter(todo => todo.id !== id));
    if (selectedTodoId === id) {
      setSelectedTodoId(null);
    }
  };

  const startEditTodo = (id: string, text: string) => {
    setEditingTodoId(id);
    setEditingText(text);
  };

  const saveTodo = () => {
    if (editingTodoId && editingText.trim()) {
      setTodos(todos.map(todo =>
        todo.id === editingTodoId ? { ...todo, text: editingText } : todo
      ));
      setEditingTodoId(null);
      setEditingText("");
    }
  };

  const selectTodo = (id: string) => {
    setSelectedTodoId(id === selectedTodoId ? null : id);
  };

  // Export/Import functions
  const handleExport = () => {
    const dataToExport = {
      version: "1.0",
      exportDate: new Date().toISOString(),
      todos: todos,
      stats: stats,
      settings: {
        darkMode: darkMode,
        selectedProject: selectedProject
      }
    };
    
    const jsonStr = JSON.stringify(dataToExport, null, 2);
    setExportData(jsonStr);
    setShowExportImport(true);
  };

  const handleCopyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(exportData);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const handleImport = () => {
    try {
      const importedData = JSON.parse(importData);
      
      // バージョンチェック
      if (importedData.version !== "1.0") {
        alert("インポートデータのバージョンが異なります");
        return;
      }
      
      // データをインポート
      if (importedData.todos) {
        setTodos(importedData.todos);
      }
      
      if (importedData.stats) {
        setStats(importedData.stats);
      }
      
      if (importedData.settings) {
        if (importedData.settings.darkMode !== undefined) {
          setDarkMode(importedData.settings.darkMode);
        }
        if (importedData.settings.selectedProject) {
          setSelectedProject(importedData.settings.selectedProject);
        }
      }
      
      setImportData("");
      setShowExportImport(false);
      alert("データをインポートしました！");
    } catch (err) {
      alert("インポートに失敗しました。正しいJSON形式か確認してください。");
      console.error('Import error:', err);
    }
  };

  // Share functions
  const generateShareUrl = () => {
    const shareData = {
      todos: todos.filter(t => !t.completed),
      createdAt: new Date().toISOString(),
      sharedBy: 'Pomodoro Flow User'
    };
    
    const encoded = btoa(encodeURIComponent(JSON.stringify(shareData)));
    const url = `${window.location.origin}${window.location.pathname}?shared=${encoded}`;
    setShareUrl(url);
    setShowShareModal(true);
  };

  const copyShareUrl = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl);
      // Show success feedback (you could add a toast notification here)
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  // Load shared data from URL
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const sharedData = urlParams.get('shared');
    
    if (sharedData) {
      try {
        const decoded = JSON.parse(decodeURIComponent(atob(sharedData)));
        if (decoded.todos && Array.isArray(decoded.todos)) {
          // Ask user if they want to import shared tasks
          if (confirm('共有されたタスクリストをインポートしますか？')) {
            const importedTodos = decoded.todos.map((todo: any) => ({
              ...todo,
              id: Date.now().toString() + Math.random(),
              createdAt: new Date()
            }));
            setTodos(prev => [...prev, ...importedTodos]);
          }
          // Clean URL
          window.history.replaceState({}, document.title, window.location.pathname);
        }
      } catch (err) {
        console.error('Failed to parse shared data:', err);
      }
    }
  }, []);

  // Template functions
  const saveAsTemplate = (todo: Todo) => {
    const template: TaskTemplate = {
      id: Date.now().toString(),
      name: todo.text.slice(0, 30),
      text: todo.text,
      priority: todo.priority,
      pomodorosEstimated: todo.pomodorosEstimated,
      tags: todo.tags,
      usageCount: 0
    };
    
    const newTemplates = [...templates, template];
    setTemplates(newTemplates);
    localStorage.setItem('templates', JSON.stringify(newTemplates));
  };

  const useTemplate = (template: TaskTemplate) => {
    const newTodo: Todo = {
      id: Date.now().toString(),
      text: template.text,
      completed: false,
      priority: template.priority,
      pomodorosCompleted: 0,
      pomodorosEstimated: template.pomodorosEstimated,
      createdAt: new Date(),
      tags: template.tags,
      order: todos.reduce((max, t) => Math.max(max, t.order || 0), 0) + 1
    };
    
    setTodos([newTodo, ...todos]);
    
    // Update usage count
    const updatedTemplates = templates.map(t => 
      t.id === template.id 
        ? { ...t, usageCount: t.usageCount + 1 }
        : t
    );
    setTemplates(updatedTemplates);
    localStorage.setItem('templates', JSON.stringify(updatedTemplates));
    
    setShowTemplates(false);
  };

  const deleteTemplate = (templateId: string) => {
    const newTemplates = templates.filter(t => t.id !== templateId);
    setTemplates(newTemplates);
    localStorage.setItem('templates', JSON.stringify(newTemplates));
  };

  // Focus mode functions
  const toggleFocusMode = () => {
    if (!focusMode) {
      // Enter focus mode
      setFocusMode(true);
      if (document.documentElement.requestFullscreen) {
        document.documentElement.requestFullscreen().catch(() => {
          // Fullscreen failed, but continue with focus mode
        });
      }
    } else {
      // Exit focus mode
      setFocusMode(false);
      if (document.fullscreenElement) {
        document.exitFullscreen().catch(() => {
          // Exit fullscreen failed
        });
      }
    }
  };

  // Drag and drop handlers
  const handleDragStart = (e: React.DragEvent, todo: Todo) => {
    setDraggedItem(todo);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent, todoId: string) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverItem(todoId);
  };

  const handleDragLeave = () => {
    setDragOverItem(null);
  };

  const handleDrop = (e: React.DragEvent, targetTodo: Todo) => {
    e.preventDefault();
    
    if (!draggedItem || draggedItem.id === targetTodo.id) {
      setDraggedItem(null);
      setDragOverItem(null);
      return;
    }

    const newTodos = [...todos];
    const draggedIndex = newTodos.findIndex(t => t.id === draggedItem.id);
    const targetIndex = newTodos.findIndex(t => t.id === targetTodo.id);

    if (draggedIndex !== -1 && targetIndex !== -1) {
      // Remove dragged item
      const [removed] = newTodos.splice(draggedIndex, 1);
      // Insert at new position
      newTodos.splice(targetIndex, 0, removed);
      
      // Update order values
      newTodos.forEach((todo, index) => {
        todo.order = index;
      });
      
      setTodos(newTodos);
    }

    setDraggedItem(null);
    setDragOverItem(null);
  };

  const handleDragEnd = () => {
    setDraggedItem(null);
    setDragOverItem(null);
  };

  // Get all unique tags from todos
  const allTags = Array.from(new Set(
    todos.flatMap(todo => todo.tags || [])
  )).sort();

  // Get tag counts
  const tagCounts = allTags.reduce((acc, tag) => {
    acc[tag] = todos.filter(todo => 
      todo.tags?.includes(tag) && !todo.completed
    ).length;
    return acc;
  }, {} as Record<string, number>);

  // Filter todos
  const filteredTodos = todos.filter(todo => {
    if (!showCompleted && todo.completed) return false;
    if (filterPriority !== "all" && todo.priority !== filterPriority) return false;
    if (searchQuery && !todo.text.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    if (selectedProject !== "all" && todo.project !== selectedProject) return false;
    if (selectedTag && !todo.tags?.includes(selectedTag)) return false;
    return true;
  });

  const activeTodos = filteredTodos.filter(t => !t.completed)
    .sort((a, b) => (a.order || 0) - (b.order || 0));
  const completedTodos = filteredTodos.filter(t => t.completed)
    .sort((a, b) => (a.order || 0) - (b.order || 0));
  
  const progress = ((currentSettings.minutes * 60 - timeLeft) / (currentSettings.minutes * 60)) * 100;

  if (!mounted) return null;

  return (
    <div className={`min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-200`}>
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-8">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 bg-gradient-to-br from-red-500 to-red-600 rounded-lg flex items-center justify-center">
                  <Timer className="w-5 h-5 text-white" />
                </div>
                <h1 className="text-xl font-semibold text-gray-900 dark:text-white">Pomodoro Flow</h1>
              </div>
              
              <nav className="hidden md:flex items-center gap-6">
                <button className="text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white font-medium text-sm">
                  Timer
                </button>
                <button className="text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white font-medium text-sm">
                  Tasks
                </button>
                <button className="text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white font-medium text-sm">
                  Analytics
                </button>
              </nav>
            </div>

            <div className="flex items-center gap-4">
              <button
                onClick={() => setShowKeyboardHelp(true)}
                className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                title="キーボードショートカット (Cmd/Ctrl + K)"
              >
                <Keyboard className="w-5 h-5" />
              </button>
              <button
                onClick={() => setShowStats(!showStats)}
                className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                title="統計グラフ (S)"
              >
                <BarChart3 className="w-5 h-5" />
              </button>
              <button
                onClick={generateShareUrl}
                className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                title="タスクリストを共有"
              >
                <Share2 className="w-5 h-5" />
              </button>
              <button
                onClick={handleExport}
                className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                title="エクスポート/インポート"
              >
                <FileJson className="w-5 h-5" />
              </button>
              <button
                onClick={() => setShowKeyboardHelp(!showKeyboardHelp)}
                className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                title="キーボードショートカット (⌘+K)"
              >
                <Keyboard className="w-5 h-5" />
              </button>
              <button
                onClick={toggleFocusMode}
                className={`p-2 rounded-lg transition-colors ${
                  focusMode 
                    ? 'text-red-600 bg-red-100 dark:bg-red-900/30 dark:text-red-400' 
                    : 'text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
                title="集中モード (F)"
              >
                {focusMode ? <Minimize2 className="w-5 h-5" /> : <Maximize2 className="w-5 h-5" />}
              </button>
              <button
                onClick={() => setDarkMode(!darkMode)}
                className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              >
                {darkMode ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
              </button>
              <button className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
                <Settings className="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Focus Mode UI */}
      {focusMode ? (
        <div className="flex flex-col items-center justify-center h-[calc(100vh-4rem)] bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800">
          <div className="text-center space-y-8 max-w-2xl mx-auto p-8">
            {/* Selected Task Display */}
            {selectedTodoId && (() => {
              const selectedTodo = todos.find(t => t.id === selectedTodoId);
              return selectedTodo ? (
                <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-6 mb-8">
                  <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                    {selectedTodo.text}
                  </h3>
                  <div className="flex items-center justify-center gap-4 text-sm text-gray-600 dark:text-gray-400">
                    <span className="flex items-center gap-1">
                      <Target className="w-4 h-4" />
                      {selectedTodo.pomodorosCompleted}/{selectedTodo.pomodorosEstimated}
                    </span>
                    {selectedTodo.tags.length > 0 && (
                      <span className="flex items-center gap-1">
                        <Hash className="w-4 h-4" />
                        {selectedTodo.tags.join(", ")}
                      </span>
                    )}
                  </div>
                </div>
              ) : null;
            })()}

            {/* Timer Display */}
            <div className="relative">
              <svg className="w-64 h-64 mx-auto transform -rotate-90">
                <circle
                  cx="128"
                  cy="128"
                  r="120"
                  stroke="currentColor"
                  strokeWidth="8"
                  fill="none"
                  className="text-gray-200 dark:text-gray-700"
                />
                <circle
                  cx="128"
                  cy="128"
                  r="120"
                  stroke="currentColor"
                  strokeWidth="8"
                  fill="none"
                  strokeDasharray={`${2 * Math.PI * 120}`}
                  strokeDashoffset={`${2 * Math.PI * 120 * (1 - progress / 100)}`}
                  className={mode === "work" ? "text-red-500" : "text-blue-500"}
                  strokeLinecap="round"
                />
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center">
                <div className="text-6xl font-bold text-gray-900 dark:text-white">
                  {Math.floor(timeLeft / 60).toString().padStart(2, "0")}:
                  {(timeLeft % 60).toString().padStart(2, "0")}
                </div>
                <div className="text-lg font-medium text-gray-600 dark:text-gray-400 mt-2">
                  {mode === "work" ? "作業中" : mode === "shortBreak" ? "短い休憩" : "長い休憩"}
                </div>
              </div>
            </div>

            {/* Timer Controls */}
            <div className="flex items-center justify-center gap-4">
              <button
                onClick={toggleTimer}
                className={`p-4 rounded-full transition-all ${
                  isRunning
                    ? "bg-red-500 hover:bg-red-600 text-white"
                    : "bg-green-500 hover:bg-green-600 text-white"
                } shadow-lg hover:shadow-xl`}
              >
                {isRunning ? <Pause className="w-8 h-8" /> : <Play className="w-8 h-8" />}
              </button>
              <button
                onClick={resetTimer}
                className="p-4 bg-gray-500 hover:bg-gray-600 text-white rounded-full shadow-lg hover:shadow-xl transition-all"
              >
                <RotateCcw className="w-8 h-8" />
              </button>
            </div>

            {/* Pomodoro Counter */}
            <div className="flex items-center justify-center gap-2">
              {[...Array(4)].map((_, i) => (
                <div
                  key={i}
                  className={`w-3 h-3 rounded-full ${
                    i < pomodoroCount ? "bg-red-500" : "bg-gray-300 dark:bg-gray-600"
                  }`}
                />
              ))}
            </div>

            {/* Exit Focus Mode Hint */}
            <div className="text-sm text-gray-500 dark:text-gray-400">
              <kbd className="px-2 py-1 bg-gray-200 dark:bg-gray-700 rounded">Esc</kbd> または <kbd className="px-2 py-1 bg-gray-200 dark:bg-gray-700 rounded">F</kbd> で集中モード終了
            </div>
          </div>
        </div>
      ) : (
      <div className="flex h-[calc(100vh-4rem)]">
        {/* Sidebar */}
        <aside className="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-y-auto">
          <div className="p-4">
            <div className="space-y-1">
              <button
                onClick={() => setSelectedProject("inbox")}
                className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                  selectedProject === "inbox"
                    ? "bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white"
                    : "text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                }`}
              >
                <FolderOpen className="w-4 h-4" />
                Inbox
                <span className="ml-auto text-xs bg-gray-200 dark:bg-gray-600 px-2 py-0.5 rounded">
                  {todos.filter(t => t.project === "inbox" && !t.completed).length}
                </span>
              </button>
              
              <button className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                <Calendar className="w-4 h-4" />
                Today
                <span className="ml-auto text-xs bg-gray-200 dark:bg-gray-600 px-2 py-0.5 rounded">
                  {activeTodos.length}
                </span>
              </button>
              
              <button className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                <Star className="w-4 h-4" />
                Important
              </button>
              
              <button className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                <Archive className="w-4 h-4" />
                Archive
              </button>
            </div>

            {/* Tags Section */}
            <div className="mt-6">
              <div className="flex items-center justify-between px-3 mb-3">
                <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                  タグ
                </h3>
                <button
                  onClick={() => setShowTagView(!showTagView)}
                  className="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
                >
                  {showTagView ? '閉じる' : 'すべて表示'}
                </button>
              </div>
              
              {showTagView ? (
                <div className="space-y-1 max-h-48 overflow-y-auto">
                  <button
                    onClick={() => setSelectedTag(null)}
                    className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm transition-colors ${
                      selectedTag === null
                        ? "bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white"
                        : "text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                    }`}
                  >
                    <span className="flex items-center gap-2">
                      <Hash className="w-3 h-3" />
                      すべてのタスク
                    </span>
                    <span className="text-xs bg-gray-200 dark:bg-gray-600 px-2 py-0.5 rounded">
                      {activeTodos.length}
                    </span>
                  </button>
                  
                  {allTags.map(tag => (
                    <button
                      key={tag}
                      onClick={() => setSelectedTag(tag === selectedTag ? null : tag)}
                      className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm transition-colors ${
                        selectedTag === tag
                          ? "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300"
                          : "text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                      }`}
                    >
                      <span className="flex items-center gap-2">
                        <Hash className="w-3 h-3" />
                        {tag}
                      </span>
                      <span className={`text-xs px-2 py-0.5 rounded ${
                        selectedTag === tag 
                          ? "bg-blue-200 dark:bg-blue-800" 
                          : "bg-gray-200 dark:bg-gray-600"
                      }`}>
                        {tagCounts[tag] || 0}
                      </span>
                    </button>
                  ))}
                </div>
              ) : (
                <div className="px-3 space-y-1">
                  {allTags.slice(0, 5).map(tag => (
                    <button
                      key={tag}
                      onClick={() => setSelectedTag(tag === selectedTag ? null : tag)}
                      className={`w-full flex items-center justify-between py-1 text-sm transition-colors rounded ${
                        selectedTag === tag
                          ? "text-blue-600 dark:text-blue-400"
                          : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                      }`}
                    >
                      <span className="flex items-center gap-2 truncate">
                        <Hash className="w-3 h-3" />
                        {tag}
                      </span>
                      <span className="text-xs">
                        {tagCounts[tag] || 0}
                      </span>
                    </button>
                  ))}
                  {allTags.length > 5 && (
                    <button
                      onClick={() => setShowTagView(true)}
                      className="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 pl-5"
                    >
                      +{allTags.length - 5} その他
                    </button>
                  )}
                </div>
              )}
            </div>

            <div className="mt-6">
              <h3 className="px-3 text-xs font-semibold text-gray-400 uppercase tracking-wider">
                Statistics
              </h3>
              <div className="mt-3 space-y-3">
                <div className="px-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600 dark:text-gray-300">Today</span>
                    <span className="font-semibold text-gray-900 dark:text-white">
                      {stats.todayPomodoros} pomodoros
                    </span>
                  </div>
                </div>
                <div className="px-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600 dark:text-gray-300">This Week</span>
                    <span className="font-semibold text-gray-900 dark:text-white">
                      {stats.weekPomodoros} pomodoros
                    </span>
                  </div>
                </div>
                <div className="px-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600 dark:text-gray-300">Completed</span>
                    <span className="font-semibold text-gray-900 dark:text-white">
                      {stats.completedTasks} tasks
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 overflow-y-auto">
          <div className="p-8">
            {/* Timer Section */}
            <div className="max-w-4xl mx-auto">
              <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 p-8 mb-8">
                {/* Timer Mode Tabs */}
                <div className="flex items-center justify-center gap-2 mb-8">
                  {(Object.keys(TIMER_SETTINGS) as TimerMode[]).map((timerMode) => (
                    <button
                      key={timerMode}
                      onClick={() => switchMode(timerMode)}
                      className={`px-4 py-2 rounded-lg font-medium text-sm transition-all ${
                        mode === timerMode
                          ? `${TIMER_SETTINGS[timerMode].bg} ${TIMER_SETTINGS[timerMode].color}`
                          : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700"
                      }`}
                    >
                      {TIMER_SETTINGS[timerMode].label}
                    </button>
                  ))}
                </div>

                {/* Selected Task Display */}
                {selectedTodo && (
                  <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <Target className="w-4 h-4 text-gray-500 dark:text-gray-400" />
                        <span className="text-sm font-medium text-gray-900 dark:text-white">
                          Current Task
                        </span>
                      </div>
                      <button
                        onClick={() => setSelectedTodoId(null)}
                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                      >
                        <X className="w-4 h-4" />
                      </button>
                    </div>
                    <p className="mt-2 text-gray-700 dark:text-gray-300">{selectedTodo.text}</p>
                    <div className="mt-2 flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400">
                      <span className="flex items-center gap-1">
                        <Timer className="w-3 h-3" />
                        {selectedTodo.pomodorosCompleted}/{selectedTodo.pomodorosEstimated} pomodoros
                      </span>
                      <span className={`flex items-center gap-1 ${PRIORITY_CONFIG[selectedTodo.priority].color}`}>
                        {React.createElement(PRIORITY_CONFIG[selectedTodo.priority].icon, { className: "w-3 h-3" })}
                        {PRIORITY_CONFIG[selectedTodo.priority].label}
                      </span>
                    </div>
                  </div>
                )}

                {/* Timer Display */}
                <div className="text-center">
                  <div className="relative inline-flex items-center justify-center">
                    <svg className="w-64 h-64 transform -rotate-90">
                      <circle
                        cx="128"
                        cy="128"
                        r="120"
                        stroke="currentColor"
                        strokeWidth="8"
                        fill="none"
                        className="text-gray-200 dark:text-gray-700"
                      />
                      <circle
                        cx="128"
                        cy="128"
                        r="120"
                        stroke="currentColor"
                        strokeWidth="8"
                        fill="none"
                        strokeDasharray={`${2 * Math.PI * 120}`}
                        strokeDashoffset={`${2 * Math.PI * 120 * (1 - progress / 100)}`}
                        className={`${currentSettings.color} transition-all duration-1000`}
                      />
                    </svg>
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                      <CurrentIcon className={`w-8 h-8 ${currentSettings.color} mb-2`} />
                      <div className="text-5xl font-bold text-gray-900 dark:text-white">
                        {formatTime(timeLeft)}
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                        {currentSettings.label}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Timer Controls */}
                <div className="flex flex-col items-center gap-4 mt-8">
                  {/* タスク未選択の警告表示 */}
                  {mode === "work" && !selectedTodoId && todos.filter(t => !t.completed).length > 0 && (
                    <div className="text-sm text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900/20 px-4 py-2 rounded-lg flex items-center gap-2">
                      <AlertCircle className="w-4 h-4" />
                      タスクを選択してください
                    </div>
                  )}
                  
                  <div className="flex items-center gap-4">
                    <button
                      onClick={toggleTimer}
                      className={`px-8 py-3 rounded-lg font-medium text-white transition-all transform hover:scale-105 ${
                        isRunning
                          ? "bg-gray-600 hover:bg-gray-700"
                          : "bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700"
                      }`}
                    >
                      {isRunning ? (
                        <span className="flex items-center gap-2">
                          <Pause className="w-5 h-5" />
                          一時停止
                        </span>
                      ) : (
                        <span className="flex items-center gap-2">
                          <Play className="w-5 h-5" />
                          開始
                        </span>
                      )}
                    </button>
                    <button
                      onClick={resetTimer}
                      className="px-6 py-3 rounded-lg font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                    >
                      <RotateCcw className="w-5 h-5" />
                    </button>
                  </div>
                </div>

                {/* Pomodoro Counter */}
                <div className="flex items-center justify-center gap-2 mt-6">
                  {[...Array(4)].map((_, i) => (
                    <div
                      key={i}
                      className={`w-2 h-2 rounded-full ${
                        i < completedPomodoros % 4
                          ? "bg-red-500"
                          : "bg-gray-300 dark:bg-gray-600"
                      }`}
                    />
                  ))}
                </div>
              </div>

              {/* Task Management Section */}
              <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 p-8">
                {/* Task Header */}
                <div className="flex items-center justify-between mb-6">
                  <h2 className="text-xl font-semibold text-gray-900 dark:text-white">Tasks</h2>
                  <div className="flex items-center gap-2">
                    <div className="relative">
                      <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                      <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search tasks..."
                        className="pl-10 pr-4 py-2 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-red-500"
                      />
                    </div>
                    <button className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
                      <Filter className="w-5 h-5" />
                    </button>
                  </div>
                </div>

                {/* Selected Tag Display */}
                {selectedTag && (
                  <div className="mb-4 flex items-center justify-between p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                    <div className="flex items-center gap-2">
                      <Hash className="w-4 h-4 text-blue-600 dark:text-blue-400" />
                      <span className="text-sm font-medium text-blue-700 dark:text-blue-300">
                        タグ: {selectedTag}
                      </span>
                      <span className="text-xs px-2 py-0.5 bg-blue-200 dark:bg-blue-800 text-blue-700 dark:text-blue-300 rounded">
                        {tagCounts[selectedTag] || 0} タスク
                      </span>
                    </div>
                    <button
                      onClick={() => setSelectedTag(null)}
                      className="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-200"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                )}

                {/* Add Task Form */}
                <div className="mb-6 space-y-2">
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={newTodo}
                      onChange={(e) => setNewTodo(e.target.value)}
                      onKeyPress={(e) => e.key === 'Enter' && addTodo()}
                      placeholder="新しいタスクを追加..."
                      className="flex-1 px-4 py-3 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-red-500"
                    />
                    <select
                      value={selectedPriority}
                      onChange={(e) => setSelectedPriority(e.target.value as Priority)}
                      className="px-3 py-3 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-red-500"
                    >
                      <option value="high">高</option>
                      <option value="medium">中</option>
                      <option value="low">低</option>
                    </select>
                    <input
                      type="number"
                      min="1"
                      max="10"
                      value={estimatedPomodoros}
                      onChange={(e) => setEstimatedPomodoros(parseInt(e.target.value) || 1)}
                      className="w-20 px-3 py-3 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white text-center focus:outline-none focus:ring-2 focus:ring-red-500"
                      placeholder="🍅"
                    />
                    <button
                      onClick={() => setShowTemplates(true)}
                      className="px-4 py-3 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white rounded-lg font-medium transition-all transform hover:scale-105"
                      title="テンプレートから追加"
                    >
                      <BookOpen className="w-5 h-5" />
                    </button>
                    <button
                      onClick={addTodo}
                      className="px-6 py-3 bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 text-white rounded-lg font-medium transition-all transform hover:scale-105"
                    >
                      <Plus className="w-5 h-5" />
                    </button>
                  </div>
                  
                  {/* タグ入力フィールド */}
                  <div className="flex items-center gap-2">
                    <Hash className="w-4 h-4 text-gray-400" />
                    <input
                      type="text"
                      value={newTodoTags}
                      onChange={(e) => setNewTodoTags(e.target.value)}
                      onKeyPress={(e) => e.key === 'Enter' && addTodo()}
                      placeholder="タグを追加 (例: 仕事, 緊急, バグ修正)"
                      className="flex-1 px-3 py-2 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-red-500"
                    />
                  </div>
                </div>

                {/* Task List */}
                <div className="space-y-2">
                  {activeTodos.length === 0 && (
                    <div className="text-center py-12">
                      <CheckCircle2 className="w-12 h-12 text-gray-300 dark:text-gray-600 mx-auto mb-3" />
                      <p className="text-gray-500 dark:text-gray-400">No tasks yet. Add one to get started!</p>
                    </div>
                  )}
                  
                  {activeTodos.map(todo => {
                    const PriorityIcon = PRIORITY_CONFIG[todo.priority].icon;
                    return (
                      <div
                        key={todo.id}
                        draggable
                        onDragStart={(e) => handleDragStart(e, todo)}
                        onDragOver={(e) => handleDragOver(e, todo.id)}
                        onDragLeave={handleDragLeave}
                        onDrop={(e) => handleDrop(e, todo)}
                        onDragEnd={handleDragEnd}
                        className={`group flex items-center gap-3 p-4 rounded-lg border transition-all cursor-move ${
                          dragOverItem === todo.id
                            ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-400 dark:border-blue-600 scale-105'
                            : selectedTodoId === todo.id 
                            ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800' 
                            : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
                        } ${draggedItem?.id === todo.id ? 'opacity-50' : ''}`}
                        onClick={() => selectTodo(todo.id)}
                      >
                        <div className="flex-shrink-0 text-gray-400 cursor-grab active:cursor-grabbing">
                          <GripVertical className="w-4 h-4" />
                        </div>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            toggleTodo(todo.id);
                          }}
                          className="text-gray-400 hover:text-green-500 transition-colors"
                        >
                          <Circle className="w-5 h-5" />
                        </button>
                        
                        <div className={`p-1 rounded ${PRIORITY_CONFIG[todo.priority].bg}`}>
                          <PriorityIcon className={`w-4 h-4 ${PRIORITY_CONFIG[todo.priority].color}`} />
                        </div>
                        
                        {editingTodoId === todo.id ? (
                          <input
                            type="text"
                            value={editingText}
                            onChange={(e) => setEditingText(e.target.value)}
                            onKeyPress={(e) => {
                              if (e.key === 'Enter') saveTodo();
                              e.stopPropagation();
                            }}
                            onBlur={saveTodo}
                            onClick={(e) => e.stopPropagation()}
                            className="flex-1 bg-transparent text-gray-900 dark:text-white outline-none"
                            autoFocus
                          />
                        ) : (
                          <div className="flex-1">
                            <div className="text-gray-900 dark:text-white font-medium">{todo.text}</div>
                            <div className="flex items-center gap-3 mt-1 flex-wrap">
                              <span className="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1">
                                <Timer className="w-3 h-3" />
                                {todo.pomodorosCompleted}/{todo.pomodorosEstimated}
                              </span>
                              {todo.tags && todo.tags.length > 0 && (
                                <div className="flex items-center gap-1 flex-wrap">
                                  {todo.tags.map((tag, index) => (
                                    <span 
                                      key={index}
                                      className="text-xs px-2 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full"
                                    >
                                      #{tag}
                                    </span>
                                  ))}
                                </div>
                              )}
                              {todo.project && (
                                <span className="text-xs text-gray-500 dark:text-gray-400">
                                  📁 {todo.project}
                                </span>
                              )}
                            </div>
                          </div>
                        )}
                        
                        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              saveAsTemplate(todo);
                            }}
                            className="p-1 text-gray-400 hover:text-blue-500 transition-colors"
                            title="テンプレートとして保存"
                          >
                            <Save className="w-4 h-4" />
                          </button>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              startEditTodo(todo.id, todo.text);
                            }}
                            className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                          >
                            <Edit2 className="w-4 h-4" />
                          </button>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              deleteTodo(todo.id);
                            }}
                            className="p-1 text-gray-400 hover:text-red-500 transition-colors"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>

                {/* Show Completed Toggle */}
                {completedTodos.length > 0 && (
                  <button
                    onClick={() => setShowCompleted(!showCompleted)}
                    className="mt-4 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 flex items-center gap-2"
                  >
                    <ChevronRight className={`w-4 h-4 transition-transform ${showCompleted ? 'rotate-90' : ''}`} />
                    Completed ({completedTodos.length})
                  </button>
                )}

                {/* Completed Tasks */}
                {showCompleted && (
                  <div className="mt-4 space-y-2 opacity-60">
                    {completedTodos.map(todo => (
                      <div
                        key={todo.id}
                        className="flex items-center gap-3 p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50"
                      >
                        <button
                          onClick={() => toggleTodo(todo.id)}
                          className="text-green-500"
                        >
                          <CheckCircle2 className="w-5 h-5" />
                        </button>
                        <div className="flex-1">
                          <div className="text-gray-500 dark:text-gray-400 line-through">{todo.text}</div>
                        </div>
                        <button
                          onClick={() => deleteTodo(todo.id)}
                          className="p-1 text-gray-400 hover:text-red-500 transition-colors"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </main>
      </div>
      )}

      {/* Keyboard Shortcuts Help Modal */}
      {showKeyboardHelp && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl max-w-2xl w-full max-h-[90vh] overflow-hidden">
            <div className="p-6 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                  <Keyboard className="w-6 h-6" />
                  キーボードショートカット
                </h2>
                <button
                  onClick={() => setShowKeyboardHelp(false)}
                  className="p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </div>

            <div className="p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
              <div className="grid gap-4">
                <div className="space-y-2">
                  <h3 className="text-sm font-semibold text-gray-900 dark:text-white uppercase tracking-wider">タイマー操作</h3>
                  <div className="space-y-1">
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">タイマー開始/停止</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">Space</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">タイマーリセット</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">R</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">作業モード</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">1</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">短い休憩</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">2</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">長い休憩</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">3</kbd>
                    </div>
                  </div>
                </div>

                <div className="space-y-2">
                  <h3 className="text-sm font-semibold text-gray-900 dark:text-white uppercase tracking-wider">タスク管理</h3>
                  <div className="space-y-1">
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">新しいタスク</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">N</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">タスク検索</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">/</kbd>
                    </div>
                  </div>
                </div>

                <div className="space-y-2">
                  <h3 className="text-sm font-semibold text-gray-900 dark:text-white uppercase tracking-wider">アプリケーション</h3>
                  <div className="space-y-1">
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">ダークモード切り替え</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">D</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">エクスポート/インポート</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">E</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">集中モード</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">F</kbd>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">ショートカットヘルプ</span>
                      <div className="flex items-center gap-1">
                        <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">⌘</kbd>
                        <span className="text-gray-500 dark:text-gray-400">+</span>
                        <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">K</kbd>
                      </div>
                    </div>
                    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50">
                      <span className="text-gray-700 dark:text-gray-300">モーダルを閉じる</span>
                      <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 dark:text-gray-100 dark:bg-gray-700 rounded">Esc</kbd>
                    </div>
                  </div>
                </div>
              </div>

              <div className="mt-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                <p className="text-sm text-blue-900 dark:text-blue-100">
                  💡 ヒント: 入力フィールドにフォーカスがある時は、ショートカットキーは無効になります。
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Export/Import Modal */}
      {showExportImport && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl max-w-3xl w-full max-h-[90vh] overflow-hidden">
            <div className="p-6 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                  <FileJson className="w-6 h-6" />
                  データのエクスポート/インポート
                </h2>
                <button
                  onClick={() => {
                    setShowExportImport(false);
                    setExportData("");
                    setImportData("");
                    setCopySuccess(false);
                  }}
                  className="p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </div>

            <div className="p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
              {/* Export Section */}
              <div className="mb-8">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                  <Download className="w-5 h-5" />
                  エクスポート
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  現在のタスク、統計、設定をJSON形式でエクスポートします。
                  コピーボタンをクリックしてクリップボードにコピーし、安全な場所に保存してください。
                </p>
                
                {exportData ? (
                  <>
                    <div className="relative">
                      <textarea
                        value={exportData}
                        readOnly
                        className="w-full h-48 p-4 bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg font-mono text-xs text-gray-800 dark:text-gray-200 resize-none"
                      />
                      <button
                        onClick={handleCopyToClipboard}
                        className={`absolute top-2 right-2 px-4 py-2 rounded-lg font-medium transition-all ${
                          copySuccess
                            ? "bg-green-500 text-white"
                            : "bg-blue-500 hover:bg-blue-600 text-white"
                        }`}
                      >
                        {copySuccess ? (
                          <span className="flex items-center gap-2">
                            <ClipboardCheck className="w-4 h-4" />
                            コピー完了！
                          </span>
                        ) : (
                          <span className="flex items-center gap-2">
                            <Copy className="w-4 h-4" />
                            クリップボードにコピー
                          </span>
                        )}
                      </button>
                    </div>
                    
                    <div className="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                      <p className="text-sm text-blue-700 dark:text-blue-300">
                        💡 ヒント: このデータをメモ帳やテキストファイルに保存しておくと、
                        後でインポート機能を使って復元できます。
                      </p>
                    </div>
                  </>
                ) : (
                  <button
                    onClick={handleExport}
                    className="px-6 py-3 bg-blue-500 hover:bg-blue-600 text-white rounded-lg font-medium transition-colors"
                  >
                    データを生成
                  </button>
                )}
              </div>

              {/* Import Section */}
              <div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                  <Upload className="w-5 h-5" />
                  インポート
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  以前エクスポートしたJSONデータを貼り付けて、タスクと設定を復元します。
                </p>
                
                <textarea
                  value={importData}
                  onChange={(e) => setImportData(e.target.value)}
                  placeholder="エクスポートしたJSONデータをここに貼り付けてください..."
                  className="w-full h-48 p-4 bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg font-mono text-xs text-gray-800 dark:text-gray-200 placeholder-gray-400 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                
                <div className="mt-4 flex items-center gap-4">
                  <button
                    onClick={handleImport}
                    disabled={!importData.trim()}
                    className={`px-6 py-3 rounded-lg font-medium transition-colors ${
                      importData.trim()
                        ? "bg-green-500 hover:bg-green-600 text-white"
                        : "bg-gray-300 dark:bg-gray-700 text-gray-500 cursor-not-allowed"
                    }`}
                  >
                    <span className="flex items-center gap-2">
                      <Upload className="w-4 h-4" />
                      データをインポート
                    </span>
                  </button>
                  
                  {importData && (
                    <button
                      onClick={() => setImportData("")}
                      className="px-4 py-3 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
                    >
                      クリア
                    </button>
                  )}
                </div>
                
                <div className="mt-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
                  <p className="text-sm text-yellow-700 dark:text-yellow-300">
                    ⚠️ 注意: インポートすると現在のデータが上書きされます。
                    必要に応じて先に現在のデータをエクスポートしてください。
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Statistics Modal */}
      {showStats && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
            <div className="p-6 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                  <BarChart3 className="w-6 h-6" />
                  統計情報
                </h2>
                <button
                  onClick={() => setShowStats(false)}
                  className="p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </div>

            <div className="p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
              {/* View Selector */}
              <div className="flex gap-2 mb-6">
                {(['daily', 'weekly', 'monthly'] as const).map(view => (
                  <button
                    key={view}
                    onClick={() => setStatsView(view)}
                    className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                      statsView === view
                        ? 'bg-red-500 text-white'
                        : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                    }`}
                  >
                    {view === 'daily' ? '日別' : view === 'weekly' ? '週別' : '月別'}
                  </button>
                ))}
              </div>

              {/* Summary Cards */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
                <div className="bg-gradient-to-br from-red-50 to-red-100 dark:from-red-900/20 dark:to-red-800/20 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Timer className="w-5 h-5 text-red-600 dark:text-red-400" />
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">今日</span>
                  </div>
                  <div className="text-2xl font-bold text-gray-900 dark:text-white">
                    {stats.todayPomodoros}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">ポモドーロ</div>
                </div>

                <div className="bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Calendar className="w-5 h-5 text-blue-600 dark:text-blue-400" />
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">今週</span>
                  </div>
                  <div className="text-2xl font-bold text-gray-900 dark:text-white">
                    {stats.weekPomodoros}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">ポモドーロ</div>
                </div>

                <div className="bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <CheckCircle2 className="w-5 h-5 text-green-600 dark:text-green-400" />
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">完了タスク</span>
                  </div>
                  <div className="text-2xl font-bold text-gray-900 dark:text-white">
                    {stats.completedTasks}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">タスク</div>
                </div>

                <div className="bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Zap className="w-5 h-5 text-purple-600 dark:text-purple-400" />
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">連続記録</span>
                  </div>
                  <div className="text-2xl font-bold text-gray-900 dark:text-white">
                    {stats.streak}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">日</div>
                </div>
              </div>

              {/* Chart */}
              <div className="bg-gray-50 dark:bg-gray-900/50 rounded-xl p-6">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                  {statsView === 'daily' ? '日別' : statsView === 'weekly' ? '週別' : '月別'}ポモドーロ実績
                </h3>
                
                {(() => {
                  const history = statsView === 'daily' 
                    ? stats.dailyHistory 
                    : statsView === 'weekly' 
                    ? stats.weeklyHistory 
                    : stats.monthlyHistory;

                  if (!history || Object.keys(history).length === 0) {
                    return (
                      <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                        データがありません
                      </div>
                    );
                  }

                  const entries = Object.entries(history).slice(-10).sort();
                  const maxValue = Math.max(...entries.map(([_, v]) => v));

                  return (
                    <div className="space-y-3">
                      {entries.map(([date, count]) => (
                        <div key={date} className="flex items-center gap-3">
                          <div className="w-24 text-sm text-gray-600 dark:text-gray-400 text-right">
                            {date}
                          </div>
                          <div className="flex-1 relative h-8 bg-gray-200 dark:bg-gray-700 rounded">
                            <div 
                              className="absolute inset-y-0 left-0 bg-gradient-to-r from-red-400 to-red-500 rounded"
                              style={{ width: `${(count / maxValue) * 100}%` }}
                            />
                            <div className="absolute inset-y-0 left-2 flex items-center text-white text-sm font-medium">
                              {count}
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  );
                })()}
              </div>

              {/* Productivity Insights */}
              <div className="mt-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                <h4 className="font-semibold text-blue-900 dark:text-blue-100 mb-2">
                  💡 生産性のヒント
                </h4>
                <ul className="text-sm text-blue-800 dark:text-blue-200 space-y-1">
                  <li>• 最も生産的な時間帯を見つけて、重要なタスクをその時間に配置しましょう</li>
                  <li>• 1日4ポモドーロを目標にして、徐々に増やしていきましょう</li>
                  <li>• 休憩時間は必ず取り、リフレッシュすることが大切です</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Templates Modal */}
      {showTemplates && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl max-w-3xl w-full max-h-[90vh] overflow-hidden">
            <div className="p-6 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                  <BookOpen className="w-6 h-6" />
                  タスクテンプレート
                </h2>
                <button
                  onClick={() => setShowTemplates(false)}
                  className="p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </div>

            <div className="p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
              {templates.length === 0 ? (
                <div className="text-center py-12">
                  <BookOpen className="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
                  <p className="text-gray-500 dark:text-gray-400 mb-2">テンプレートがありません</p>
                  <p className="text-sm text-gray-400 dark:text-gray-500">
                    タスクの保存ボタンからテンプレートを作成できます
                  </p>
                </div>
              ) : (
                <div className="grid gap-3">
                  {templates
                    .sort((a, b) => b.usageCount - a.usageCount)
                    .map(template => (
                      <div
                        key={template.id}
                        className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-900/70 transition-colors"
                      >
                        <div className="flex-1">
                          <div className="font-medium text-gray-900 dark:text-white mb-1">
                            {template.text}
                          </div>
                          <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                            <span className="flex items-center gap-1">
                              <Flag className="w-3 h-3" />
                              {template.priority === 'high' ? '高' : template.priority === 'medium' ? '中' : '低'}
                            </span>
                            <span className="flex items-center gap-1">
                              <Timer className="w-3 h-3" />
                              {template.pomodorosEstimated}
                            </span>
                            {template.tags.length > 0 && (
                              <span className="flex items-center gap-1">
                                <Hash className="w-3 h-3" />
                                {template.tags.join(', ')}
                              </span>
                            )}
                            <span className="flex items-center gap-1">
                              <TrendingUp className="w-3 h-3" />
                              使用回数: {template.usageCount}
                            </span>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <button
                            onClick={() => useTemplate(template)}
                            className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg font-medium transition-colors"
                          >
                            使用
                          </button>
                          <button
                            onClick={() => deleteTemplate(template.id)}
                            className="p-2 text-gray-400 hover:text-red-500 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                          >
                            <Trash2 className="w-5 h-5" />
                          </button>
                        </div>
                      </div>
                    ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Share Modal */}
      {showShareModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl max-w-2xl w-full">
            <div className="p-6 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                  <Share2 className="w-6 h-6" />
                  タスクリストを共有
                </h2>
                <button
                  onClick={() => {
                    setShowShareModal(false);
                    setShareUrl("");
                  }}
                  className="p-2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </div>

            <div className="p-6">
              <p className="text-gray-600 dark:text-gray-400 mb-4">
                以下のURLをコピーして、タスクリストを他の人と共有できます。
                共有される内容は未完了のタスクのみです。
              </p>
              
              <div className="flex gap-2">
                <input
                  type="text"
                  value={shareUrl}
                  readOnly
                  className="flex-1 px-4 py-3 bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-900 dark:text-white font-mono text-sm"
                  onClick={(e) => e.currentTarget.select()}
                />
                <button
                  onClick={copyShareUrl}
                  className="px-4 py-3 bg-blue-500 hover:bg-blue-600 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
                >
                  <Copy className="w-5 h-5" />
                  コピー
                </button>
              </div>

              <div className="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                <p className="text-sm text-blue-800 dark:text-blue-200">
                  💡 ヒント: 共有されたリンクを開くと、タスクをインポートするか確認されます。
                  チーム内でタスクリストを簡単に共有できます。
                </p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}