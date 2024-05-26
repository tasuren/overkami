import "./App.css";

import { ScreenProvider } from "./screen";
import { Menu } from "./ui/Menu";

function App() {
    return (
        <ScreenProvider>
            <div class="pt-[32px] flex flex-col space-y-2">
                <Menu className="w-fit h-auto text-sm" />
            </div>
        </ScreenProvider>
    );
}

export default App;
