import { ScreenProvider } from "./screen";
import { Menu } from "./ui/Menu";

function App() {
    return (
        <ScreenProvider>
            <div class="pt-[32px] mx-auto flex flex-col space-y-2">
                <Menu className="w-fit" />
            </div>
        </ScreenProvider>
    );
}

export default App;
