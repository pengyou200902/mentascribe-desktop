import type { DashboardPage } from '../../types';

interface SidebarProps {
  currentPage: DashboardPage;
  onNavigate: (page: DashboardPage) => void;
}

interface NavItem {
  id: DashboardPage;
  label: string;
  icon: JSX.Element;
}

// Icons with refined stroke design
const HomeIcon = ({ active }: { active?: boolean }) => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={active ? 2 : 1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
  </svg>
);

const HistoryIcon = ({ active }: { active?: boolean }) => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={active ? 2 : 1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

const DictionaryIcon = ({ active }: { active?: boolean }) => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={active ? 2 : 1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25" />
  </svg>
);

const SettingsIcon = ({ active }: { active?: boolean }) => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={active ? 2 : 1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
    <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
  </svg>
);

const HelpIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M9.879 7.519c1.171-1.025 3.071-1.025 4.242 0 1.172 1.025 1.172 2.687 0 3.712-.203.179-.43.326-.67.442-.745.361-1.45.999-1.45 1.827v.75M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9 5.25h.008v.008H12v-.008z" />
  </svg>
);

// Microphone logo with waveform
const LogoIcon = () => (
  <div className="relative flex items-center gap-1">
    <svg className="w-7 h-7" viewBox="0 0 24 24" fill="none">
      <path
        d="M12 15.75a3.75 3.75 0 003.75-3.75V6a3.75 3.75 0 00-7.5 0v6a3.75 3.75 0 003.75 3.75z"
        className="fill-amber-500/20 dark:fill-amber-400/20"
        stroke="currentColor"
        strokeWidth={1.5}
      />
      <path
        d="M12 15.75a3.75 3.75 0 003.75-3.75V6a3.75 3.75 0 00-7.5 0v6a3.75 3.75 0 003.75 3.75zM18.75 10.5v1.5a6.75 6.75 0 01-13.5 0v-1.5M12 18.75v3M9 21.75h6"
        stroke="currentColor"
        strokeWidth={1.5}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
    {/* Decorative waveform bars */}
    <div className="flex items-center gap-0.5 opacity-60">
      <div className="w-0.5 h-2 rounded-full bg-amber-500 dark:bg-amber-400" />
      <div className="w-0.5 h-3 rounded-full bg-amber-500 dark:bg-amber-400" />
      <div className="w-0.5 h-2 rounded-full bg-amber-500 dark:bg-amber-400" />
    </div>
  </div>
);

const navItems: NavItem[] = [
  { id: 'home', label: 'Home', icon: <HomeIcon /> },
  { id: 'dictionary', label: 'Dictionary', icon: <DictionaryIcon /> },
  { id: 'history', label: 'History', icon: <HistoryIcon /> },
];

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="w-60 bg-stone-50 dark:bg-stone-900 border-r border-stone-200 dark:border-stone-800 flex flex-col h-full transition-colors duration-200">
      {/* Logo */}
      <div className="px-5 py-5">
        <div className="flex items-center gap-2.5">
          <LogoIcon />
          <div className="flex flex-col">
            <span className="text-lg font-semibold tracking-tight text-stone-900 dark:text-stone-100">
              MentaScribe
            </span>
            <span className="text-2xs text-stone-400 dark:text-stone-500 -mt-0.5 font-medium tracking-wide">
              Voice to Text
            </span>
          </div>
        </div>
      </div>

      {/* Main Navigation */}
      <nav className="flex-1 px-3 py-4">
        <div className="space-y-1">
          {navItems.map((item) => {
            const isActive = currentPage === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onNavigate(item.id)}
                className={`
                  group w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium
                  transition-all duration-200
                  ${isActive
                    ? 'bg-amber-500/10 dark:bg-amber-400/10 text-amber-700 dark:text-amber-400 shadow-sm shadow-amber-500/5'
                    : 'text-stone-600 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-800/50 hover:text-stone-900 dark:hover:text-stone-200'
                  }
                `}
              >
                <span className={`transition-colors duration-200 ${isActive ? 'text-amber-600 dark:text-amber-400' : 'text-stone-400 dark:text-stone-500 group-hover:text-stone-600 dark:group-hover:text-stone-400'}`}>
                  {item.id === 'home' && <HomeIcon active={isActive} />}
                  {item.id === 'dictionary' && <DictionaryIcon active={isActive} />}
                  {item.id === 'history' && <HistoryIcon active={isActive} />}
                </span>
                {item.label}
                {isActive && (
                  <div className="ml-auto w-1.5 h-1.5 rounded-full bg-amber-500 dark:bg-amber-400" />
                )}
              </button>
            );
          })}
        </div>
      </nav>

      {/* Bottom Navigation */}
      <div className="px-3 py-4 border-t border-stone-200 dark:border-stone-800">
        <div className="space-y-1">
          <button
            onClick={() => onNavigate('settings')}
            className={`
              group w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium
              transition-all duration-200
              ${currentPage === 'settings'
                ? 'bg-amber-500/10 dark:bg-amber-400/10 text-amber-700 dark:text-amber-400 shadow-sm shadow-amber-500/5'
                : 'text-stone-600 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-800/50 hover:text-stone-900 dark:hover:text-stone-200'
              }
            `}
          >
            <span className={`transition-colors duration-200 ${currentPage === 'settings' ? 'text-amber-600 dark:text-amber-400' : 'text-stone-400 dark:text-stone-500 group-hover:text-stone-600 dark:group-hover:text-stone-400'}`}>
              <SettingsIcon active={currentPage === 'settings'} />
            </span>
            Settings
          </button>
          <button className="group w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium text-stone-600 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-800/50 hover:text-stone-900 dark:hover:text-stone-200 transition-all duration-200">
            <span className="text-stone-400 dark:text-stone-500 group-hover:text-stone-600 dark:group-hover:text-stone-400 transition-colors duration-200">
              <HelpIcon />
            </span>
            Help
          </button>
        </div>
      </div>

      {/* Version */}
      <div className="px-5 py-3 text-2xs text-stone-400 dark:text-stone-600">
        v1.0.0
      </div>
    </aside>
  );
}
