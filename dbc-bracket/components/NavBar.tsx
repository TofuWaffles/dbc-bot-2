import Link from "next/link";


const Navbar = () => {
    return (
        <nav className="bg-black w-full py-5">
            <Link href="/"><p className="w-full text-center text-white font-bold text-2xl">Discord Brawl Cup</p></Link>
        </nav>
    );
};

export default Navbar;